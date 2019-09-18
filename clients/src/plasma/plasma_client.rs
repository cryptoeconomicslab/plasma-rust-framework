use super::command::{Command, NewTransactionEvent};
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
use ovm::db::{RangeAtBlockDb, SignedByDb, TransactionDb};
use ovm::deciders::SignVerifier;
use ovm::property_executor::PropertyExecutor;
use ovm::types::{Integer, Property, PropertyInput, StateUpdate};
use ovm::DeciderManager;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::{Range, Transaction, TransactionParams};
use plasma_db::impls::kvs::CoreDbLevelDbImpl;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use pubsub_messaging::{connect, Client as PubsubClient, ClientHandler, Message, Sender};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

pub struct PlasmaClientShell {
    aggregator_endpoint: String,
    commitment_contract_address: Address,
    private_key: String,
    my_address: Address,
    controller: Option<PlasmaClientController>,
}

impl PlasmaClientShell {
    pub fn new(
        aggregator_endpoint: String,
        commitment_contract_address: Address,
        private_key: &str,
    ) -> Self {
        let raw_key = hex::decode(private_key).unwrap();
        let secret_key = SecretKey::from_raw(&raw_key).unwrap();
        let my_address: Address = secret_key.public().address().into();
        Self {
            aggregator_endpoint,
            commitment_contract_address,
            private_key: private_key.to_string(),
            my_address,
            controller: None,
        }
    }

    /// Claim for ownership
    pub fn create_ownership_state_object(to_address: Address) -> Property {
        /*
         * There exists tx such that state_update.deprecate(tx):
         *   SignedBy(tx, to_address).
         */
        DeciderManager::there_exists_such_that(vec![
            PropertyInput::ConstantProperty(DeciderManager::q_tx(vec![
                PropertyInput::Placeholder(Bytes::from("state_update")),
            ])),
            PropertyInput::ConstantBytes(Bytes::from("tx")),
            PropertyInput::ConstantProperty(DeciderManager::signed_by_decider(vec![
                PropertyInput::ConstantAddress(to_address),
                PropertyInput::Placeholder(Bytes::from("tx")),
            ])),
        ])
    }

    // Claim for checkpoint
    pub fn create_checkpoint_property(specified_block_number: Integer, range: Range) -> Property {
        /*
         * For all b such that b < specified_block_number:
         *   For all state_update such that block_range_quantifier(b, range):
         *     IsDeprecated(state_update).
         */
        DeciderManager::for_all_such_that_decider(
            // less than quantifier
            DeciderManager::q_less_than(vec![PropertyInput::ConstantInteger(
                specified_block_number,
            )]),
            Bytes::from("block"),
            DeciderManager::for_all_such_that_decider(
                // block range quantifier
                DeciderManager::q_block(vec![
                    PropertyInput::Placeholder(Bytes::from("block")),
                    PropertyInput::ConstantRange(range),
                ]),
                Bytes::from("state_update"),
                DeciderManager::is_deprecated(vec![PropertyInput::Placeholder(Bytes::from(
                    "state_update",
                ))]),
            ),
        )
    }

    pub fn connect(&mut self) {
        let plasma_client =
            PlasmaClient::<CoreDbLevelDbImpl>::new(Address::zero(), self.private_key.clone());
        let controller = PlasmaClientController::new(plasma_client);
        let pubsub_client = connect(self.aggregator_endpoint.clone(), controller.clone()).unwrap();
        self.controller = Some(controller.clone_by_pubsub_client(pubsub_client));
        let abi: Vec<Event> = vec![Event {
            name: "BlockSubmitted".to_owned(),
            inputs: vec![
                EventParam {
                    name: "blockNumber".to_owned(),
                    kind: ParamType::Uint(64),
                    indexed: false,
                },
                EventParam {
                    name: "root".to_owned(),
                    kind: ParamType::FixedBytes(32),
                    indexed: false,
                },
            ],
            anonymous: false,
        }];
        let kvs = CoreDbLevelDbImpl::open("eventdb");
        let db = EventDbImpl::from(kvs);
        let watcher = EventWatcher::new(
            "http://localhost:8545",
            self.commitment_contract_address,
            abi,
            db,
            self.controller.clone().unwrap(),
        );
        tokio::spawn(watcher);
    }
    pub fn send_transaction(&self, to_address: &str, start: u64, end: u64) {
        let to_address = Address::from_slice(&hex::decode(to_address).unwrap());
        let controller = self.controller.clone().unwrap();
        let tx = controller.plasma_client.lock().unwrap().create_transaction(
            Range::new(start, end),
            Bytes::from(Self::create_ownership_state_object(to_address).to_abi()),
        );
        println!("{:?}", tx);
        let command = Command {
            command_type: Integer(0),
            body: Bytes::from(tx.to_abi()),
        };
        let mut pubsub_client = controller.pubsub_client.clone().unwrap();
        let msg = Message::new("Aggregator".to_string(), command.to_abi());
        pubsub_client.send(msg);
    }
    pub fn initialize(&self) {
        let controller = self.controller.clone().unwrap();
        //        controller.fetch_block(Integer(0));
        controller.initialize()
    }
    pub fn get_balance(&self) -> u64 {
        let controller = self.controller.clone().unwrap();
        let plasma_client = controller.plasma_client.lock().unwrap();
        plasma_client
            .get_state_updates()
            .iter()
            .filter(|s| {
                let p = &s.get_property().inputs[2];
                if let PropertyInput::ConstantProperty(signed_by) = p {
                    if let PropertyInput::ConstantAddress(address) = signed_by.inputs[0] {
                        return address == self.my_address
                    }
                }
                false
            })
            .fold(0, |acc, s| {
                acc + s.get_range().get_end() - s.get_range().get_start()
            })
    }
}

#[derive(Clone)]
pub struct PlasmaClientController {
    pub plasma_client: Arc<Mutex<PlasmaClient<CoreDbLevelDbImpl>>>,
    pub pubsub_client: Option<PubsubClient>,
}

impl PlasmaClientController {
    pub fn new(plasma_client: PlasmaClient<CoreDbLevelDbImpl>) -> Self {
        Self {
            plasma_client: Arc::new(Mutex::new(plasma_client)),
            pubsub_client: None,
        }
    }
    fn clone_by_pubsub_client(&self, pubsub_client: PubsubClient) -> Self {
        PlasmaClientController {
            plasma_client: self.plasma_client.clone(),
            pubsub_client: Some(pubsub_client),
        }
    }
    fn fetch_block(&self, block_number: Integer) {
        let command = Command::create_fetch_block_request(block_number);
        let msg = Message::new("Aggregator".to_string(), command.to_abi());
        let mut pubsub_client = self.pubsub_client.clone().unwrap();
        pubsub_client.send(msg);
    }
    fn initialize(&self) {
        let mut plasma_client = self.plasma_client.lock().unwrap();
        plasma_client.insert_test_ranges()
    }
}

impl ClientHandler for PlasmaClientController {
    fn handle_message(&self, msg: Message, _sender: Sender) {
        let plasma_client = self.plasma_client.lock().unwrap();
        let command = Command::from_abi(&msg.message).unwrap();
        if command.command_type.0 == 3 {
            plasma_client.handle_new_block(PlasmaBlock::from_abi(&command.body).unwrap());
        } else if command.command_type.0 == 4 {
            plasma_client
                .handle_new_transaction(&NewTransactionEvent::from_abi(&command.body).unwrap());
        } else {
            println!("undefined command type {:?}", command.command_type.0);
        }
    }
}

impl EventHandler for PlasmaClientController {
    fn on_event(&self, log: &Log) {
        let decoded_param = log.params.first().unwrap();
        println!(
            "block number is {:?}",
            decoded_param.token.clone().to_uint().unwrap()
        );
        let block_number = decoded_param.token.clone().to_uint().unwrap();
        self.fetch_block(Integer(block_number.as_u64()));
    }
}

/// Plasma Client on OVM.
pub struct PlasmaClient<KVS: KeyValueStore> {
    plasma_contract_address: Address,
    secret_key: SecretKey,
    my_address: Address,
    decider: PropertyExecutor<KVS>,
}

impl<KVS: KeyValueStore + DatabaseTrait> PlasmaClient<KVS> {
    pub fn new(plasma_contract_address: Address, private_key: String) -> Self {
        let raw_key = hex::decode(private_key).unwrap();
        let secret_key = SecretKey::from_raw(&raw_key).unwrap();
        let my_address: Address = secret_key.public().address().into();

        PlasmaClient {
            plasma_contract_address,
            secret_key,
            my_address,
            decider: Default::default(),
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
        PlasmaClientShell::create_checkpoint_property(block_number, range)
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
    pub fn handle_new_block(&self, mut block: PlasmaBlock) {
        println!("handle_new_block {:?}", block.get_block_number());
        // println!("handle_new_block {:?} {:?}", block.get_block_number(), block.get_state_updates());
        let range_db = self.decider.get_range_db();
        let range_at_block_db = RangeAtBlockDb::new(range_db);
        let transaction_db = TransactionDb::new(self.decider.get_range_db());
        let signed_by_db = SignedByDb::new(self.decider.get_db());
        let root = block.merkelize().unwrap();

        for s in block.get_state_updates().iter() {
            assert!(range_at_block_db
                .store_witness(
                    block.get_inclusion_proof(s.clone()).unwrap(),
                    block
                        .get_plasma_data_block(root.clone(), s.clone())
                        .unwrap(),
                )
                .is_ok());
        }
        for tx in block.get_transactions().iter() {
            transaction_db.put_transaction(tx.prev_state_block_number.0, tx.transaction.clone());
            let message = Bytes::from(tx.transaction.to_body_abi());
            signed_by_db.store_witness(
                SignVerifier::recover(tx.transaction.get_signature(), &message),
                message,
                tx.transaction.get_signature().clone(),
            );
        }
        for su in self.get_state_updates() {
            let property = PlasmaClientShell::create_checkpoint_property(
                Integer(su.get_block_number().0),
                su.get_range(),
            );
            let decision = self.decider.decide(&property);
            println!(
                "decide local checkpoint claim {:?}. decision = {:?}",
                su.get_range(),
                decision.is_ok()
            );
        }
        self.update_state_updates(block.get_state_updates().to_vec());
    }

    pub fn handle_new_transaction(&self, event: &NewTransactionEvent) {
        println!("handle_new_transaction");
        // println!("handle_new_transaction {:?}", event);
        let transaction_db = TransactionDb::new(self.decider.get_range_db());
        transaction_db.put_transaction(event.prev_state_block_number.0, event.transaction.clone());
    }

    pub fn insert_test_ranges(&mut self) {
        let ownership_decider_id = DeciderManager::get_decider_address(9);
        let mut state_updates = vec![];
        for i in 0..3 {
            state_updates.push(StateUpdate::new(
                Integer::new(0),
                Range::new(i * 20, (i + 1) * 20),
                PlasmaClientShell::create_ownership_state_object(Address::from_slice(
                    &hex::decode("627306090abab3a6e1400e9345bc60c78a8bef57").unwrap(),
                ))
            ));
        }
        let plasma_block = PlasmaBlock::new(0, state_updates, vec![]);
        self.handle_new_block(plasma_block);
    }

    pub fn get_state_updates(&self) -> Vec<StateUpdate> {
        let range_db = self.decider.get_range_db();
        let state_db = StateDb::new(range_db);
        state_db.get_all_state_updates().unwrap_or_else(|_| vec![])
    }

    pub fn update_state_updates(&self, state_updates: Vec<StateUpdate>) {
        let range_db = self.decider.get_range_db();
        let mut state_db = StateDb::new(range_db);

        for s in state_updates.iter() {
            let _ = state_db.put_verified_state_update(s.clone());
        }
    }
}
