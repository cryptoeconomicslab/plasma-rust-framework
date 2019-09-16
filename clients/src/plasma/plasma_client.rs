use super::plasma_block::PlasmaBlock;
use super::state_db::StateDb;
use bytes::Bytes;
use contract_wrapper::plasma_contract_adaptor::PlasmaContractAdaptor;
use ethabi::Contract as ContractABI;
use ethabi::{Event, EventParam, ParamType};
use ethereum_types::Address;
use ethsign::SecretKey;
use event_watcher::event_db::EventDbImpl;
use event_watcher::event_watcher::{EventHandler, EventWatcher, Log};
use futures::{future, Async, Future, Poll, Stream};
use ovm::deciders::SignVerifier;
use ovm::statements::create_plasma_property;
use ovm::types::{Integer, Property, PropertyInput, StateUpdate};
use ovm::DeciderManager;
use plasma_core::data_structure::abi::Encodable;
use plasma_core::data_structure::{Range, Transaction, TransactionParams};
use plasma_db::impls::kvs::CoreDbMemoryImpl;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::RangeDbImpl;
use pubsub_messaging::{connect, Client as PubsubClient, ClientHandler, Message, Sender};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use tokio::timer::Interval;

pub struct PlasmaClientShell {
    aggregator_endpoint: String,
    commitment_contract_address: Address,
    private_key: String,
    controller: Option<PlasmaClientController>,
    pubsub_client: Option<PubsubClient>,
}

impl PlasmaClientShell {
    pub fn new(aggregator_endpoint: String, commitment_contract_address: Address, private_key: &str) -> Self {
        Self {
            aggregator_endpoint,
            commitment_contract_address,
            private_key: private_key.to_string(),
            controller: None,
            pubsub_client: None,
        }
    }
    fn create_ownership_state_object(to_address: Address) -> Property {
        let ownership_decider_id = DeciderManager::get_decider_address(9);
        Property::new(
            ownership_decider_id,
            vec![
                PropertyInput::Placeholder(Bytes::from("state_update")),
                PropertyInput::ConstantAddress(to_address),
            ],
        )
    }
    pub fn connect(&mut self) {
        let plasma_client = PlasmaClient::<CoreDbMemoryImpl>::new(
            Address::zero(),
            self.aggregator_endpoint.clone(),
            self.private_key.clone(),
        );
        let controller = PlasmaClientController::new(plasma_client);
        self.controller = Some(controller.clone());
        let abi: Vec<Event> = vec![Event {
            name: "BlockSubmitted".to_owned(),
            inputs: vec![EventParam {
                name: "blockNumber".to_owned(),
                kind: ParamType::Uint(64),
                indexed: false,
            },EventParam {
                name: "root".to_owned(),
                kind: ParamType::FixedBytes(32),
                indexed: false,
            }],
            anonymous: false,
        }];
        let kvs = CoreDbMemoryImpl::open("kvs");
        let db = EventDbImpl::from(kvs);
        let watcher = EventWatcher::new("http://localhost:8545", self.commitment_contract_address, abi, db, controller.clone());
        tokio::spawn(watcher);
    }
    pub fn send_transaction(&self, to_address: &str, start: u64, end: u64) {
        let to_address = Address::from_slice(&hex::decode(to_address).unwrap());
        let controller = self.controller.clone().unwrap();
        let tx = controller.plasma_client.lock().unwrap().create_transaction(
            Range::new(start, end),
            Bytes::from(Self::create_ownership_state_object(to_address).to_abi()),
        );
        let mut pubsub_client =
            connect(self.aggregator_endpoint.clone(), controller.clone()).unwrap();
        println!("{:?}", tx);
        let msg = Message::new("Aggregator".to_string(), tx.to_abi());
        pubsub_client.send(msg);
        assert!(pubsub_client.handle.join().is_ok());
    }
}

#[derive(Clone)]
pub struct PlasmaClientController {
    pub plasma_client: Arc<Mutex<PlasmaClient<CoreDbMemoryImpl>>>,
}

impl PlasmaClientController {
    pub fn new(plasma_client: PlasmaClient<CoreDbMemoryImpl>) -> Self {
        Self {
            plasma_client: Arc::new(Mutex::new(plasma_client)),
        }
    }
}

impl ClientHandler for PlasmaClientController {
    fn handle_message(&self, msg: Message, _sender: Sender) {
        println!("ClientHandler handle_message: {:?}", msg);
    }
}

impl EventHandler for PlasmaClientController {
    fn on_event(&self, log: &Log) {
        println!("event > {:?}", log.event_signature);
        // event > 0x90890809c654f11d6e72a28fa60149770a0d11ec6c92319d6ceb2bb0a4ea1a15

        let decoded_param = log.params.first().unwrap();
        println!(
            "param > {:?}: {:?}",
            decoded_param.event_param.name,
            decoded_param.token.clone().to_uint().unwrap()
        );
        // param > "value": 22469980537774239738630940880827529904616858526135975343779764542717423171395
        // self.send_transaction("", 0, 1);
        let mut pubsub_client =
            connect("127.0.0.1:8080".to_string(), self.clone()).unwrap();
        let msg = Message::new("Aggregator".to_string(), b"fetch".to_vec());
        pubsub_client.send(msg);
        assert!(pubsub_client.handle.join().is_ok());

    }
}

/// Plasma Client on OVM.
pub struct PlasmaClient<KVS> {
    plasma_contract_address: Address,
    range_db: RangeDbImpl<KVS>,
    secret_key: SecretKey,
    aggregator_endpoint: String,
    my_address: Address,
}

impl<KVS: KeyValueStore + DatabaseTrait> PlasmaClient<KVS> {
    pub fn new(
        plasma_contract_address: Address,
        aggregator_endpoint: String,
        private_key: String,
    ) -> Self {
        let raw_key = hex::decode(private_key).unwrap();
        let secret_key = SecretKey::from_raw(&raw_key).unwrap();
        let my_address: Address = secret_key.public().address().into();
        let kvs = KVS::open("kvs");
        let range_db = RangeDbImpl::from(kvs);

        PlasmaClient {
            plasma_contract_address,
            range_db,
            secret_key,
            my_address,
            aggregator_endpoint,
        }
    }

    /// Deposit to plasma contract
    /// Send ethereum transaction to Plasma Deposit Contract.
    /// amount: amount to deposit
    /// property: initial state object
    pub fn deposit(&self, amount: u64, property: Property) {
        // TODO: add PlasmaContractABI
        let f = File::open("PlasmaContract.json").unwrap();
        let reader = BufReader::new(f);
        let contract_abi = ContractABI::load(reader).unwrap();
        let plasma_contract = PlasmaContractAdaptor::new(
            "http://127.0.0.1:9545",
            &self.plasma_contract_address.to_string(),
            contract_abi,
        )
        .unwrap();
        // TODO: handle result
        let _result = plasma_contract.deposit(self.my_address, amount, property);
    }

    /// Create transaction to update state for specific coin range.
    /// TODO: maybe need to specify Property for how state transition works.
    pub fn create_transaction(&self, range: Range, parameters: Bytes) -> Transaction {
        let transaction_params =
            TransactionParams::new(self.plasma_contract_address, range, parameters);

        let signature =
            SignVerifier::sign(&self.secret_key, &Bytes::from(transaction_params.to_abi()));
        Transaction::from_params(transaction_params, signature)
    }

    /// Start exit on plasma. return exit property
    pub fn get_exit_claim(&self, block_number: Integer, range: Range) -> Property {
        // TODO: decide property and claim property to contract
        // TODO: store as exit list
        create_plasma_property(block_number, range)
    }

    /// Handle exit on plasma.
    /// After dispute period, withdraw from Plasma Contract.
    pub fn finalize_exit(&self, state_update: StateUpdate, range: Range) {
        // TODO: add PlasmaContractABI
        let f = File::open("PlasmaContract.json").unwrap();
        let reader = BufReader::new(f);
        let contract_abi = ContractABI::load(reader).unwrap();
        let plasma_contract = PlasmaContractAdaptor::new(
            "http://127.0.0.1:9545",
            &self.plasma_contract_address.to_string(),
            contract_abi,
        )
        .unwrap();

        // TODO: create checkpoint struct
        // TODO: decide check point is exitable
        let checkpoint = (state_update, range);

        // TODO: handle result
        let _result = plasma_contract.withdraw(self.my_address, checkpoint);
    }

    /// Challenge to specific exit by claiming contradicting statement.
    pub fn challenge(&self) {}

    /// Handle BlockSubmitted Event from aggregator
    /// check new state update and verify, store them.
    pub fn handle_new_block(&self, _block: PlasmaBlock) {}

    pub fn get_state_updates(&self) -> Vec<StateUpdate> {
        let state_db = StateDb::new(&self.range_db);
        state_db.get_all_state_updates().unwrap_or_else(|_| vec![])
    }

    pub fn update_state_updates(&self, state_updates: Vec<StateUpdate>) {
        let mut state_db = StateDb::new(&self.range_db);

        for s in state_updates.iter() {
            let _ = state_db.put_verified_state_update(s.clone());
        }
    }
}
