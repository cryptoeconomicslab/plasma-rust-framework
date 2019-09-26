use super::command::{Command, NewTransactionEvent};
use super::plasma_block::PlasmaBlock;
use super::state_db::StateDb;
use super::token::Token;
use super::utils::string_to_address;
use super::wallet_manager::WalletManager;
use abi_utils::{Decodable, Encodable};
use bytes::Bytes;
use contract_wrapper::plasma_contract_adaptor::PlasmaContractAdaptor;
use ethabi::Contract as ContractABI;
use ethabi::{Event, EventParam, ParamType};
use ethereum_types::Address;
use ethsign::SecretKey;
use event_watcher::event_db::EventDbImpl;
use event_watcher::event_watcher::{EventHandler, EventWatcher, Log};
use ovm::db::{RangeAtBlockDb, SignedByDb, TransactionDb, TransactionFilterBuilder};
use ovm::deciders::SignVerifier;
use ovm::property_executor::PropertyExecutor;
use ovm::types::{Integer, Property, PropertyInput, StateUpdate};
use ovm::DeciderManager;
use plasma_core::data_structure::{Metadata, Range, Transaction, TransactionParams};
use plasma_db::impls::kvs::CoreDbLevelDbImpl;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use pubsub_messaging::{connect, Client as PubsubClient, ClientHandler, Message, Sender};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

pub struct PlasmaClientShell {
    aggregator_endpoint: String,
    commitment_contract_address: Address,
    controller: Option<PlasmaClientController>,
}

impl PlasmaClientShell {
    pub fn new(aggregator_endpoint: String, commitment_contract_address: Address) -> Self {
        Self {
            aggregator_endpoint,
            commitment_contract_address,
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

    /// Claim for channel
    pub fn create_channel_state_object(
        my_address: Address,
        counter_party_address: Address,
    ) -> Property {
        /*
         * There exists tx such that state_update.is_same_coin_range(tx):
         *   SignedBy(tx, to_address) and SignedBy(tx, counter_party_address)
         */
        DeciderManager::there_exists_such_that(vec![
            PropertyInput::ConstantProperty(DeciderManager::q_tx(vec![
                PropertyInput::Placeholder(Bytes::from("state_update")),
            ])),
            PropertyInput::ConstantBytes(Bytes::from("tx")),
            PropertyInput::ConstantProperty(DeciderManager::and_decider(
                DeciderManager::signed_by_decider(vec![
                    PropertyInput::ConstantAddress(my_address),
                    PropertyInput::Placeholder(Bytes::from("tx")),
                ]),
                DeciderManager::signed_by_decider(vec![
                    PropertyInput::ConstantAddress(counter_party_address),
                    PropertyInput::Placeholder(Bytes::from("tx")),
                ]),
            )),
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

    pub fn get_my_address(&self, session: &Bytes) -> Option<Address> {
        let controller = self.controller.clone().unwrap();
        let plasma_client = controller.plasma_client.lock().unwrap();
        plasma_client.get_my_address(session)
    }

    pub fn connect(&mut self) {
        let plasma_client = PlasmaClient::<CoreDbLevelDbImpl>::new(Address::zero());
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
    pub fn search_range(&self, deposit_contract_address: Address, amount: u64) -> Option<Range> {
        self.controller
            .clone()
            .unwrap()
            .search_range(deposit_contract_address, amount)
    }
    /// Creates new account
    pub fn create_account(&self) -> (Bytes, SecretKey) {
        let controller = self.controller.clone().unwrap();
        let plasma_client = controller.plasma_client.lock().unwrap();
        plasma_client.create_account()
    }
    /// Imports secret key
    pub fn import_account(&self, private_key: &str) -> (Bytes, SecretKey) {
        let raw_key = hex::decode(private_key).unwrap();
        let controller = self.controller.clone().unwrap();
        let plasma_client = controller.plasma_client.lock().unwrap();
        plasma_client.import_key(&raw_key)
    }
    pub fn send_transaction(
        &self,
        session: &Bytes,
        deposit_contract_address: Option<Address>,
        start: u64,
        end: u64,
        state_object: Property,
        metadata: Metadata,
    ) {
        let deposit_contract_address = deposit_contract_address.unwrap_or_else(Address::zero);
        let controller = self.controller.clone().unwrap();
        let tx = controller.plasma_client.lock().unwrap().create_transaction(
            session,
            deposit_contract_address,
            Range::new(start, end),
            Bytes::from(state_object.to_abi()),
            metadata,
        );
        let command = Command {
            command_type: Integer(0),
            body: Bytes::from(tx.to_abi()),
        };
        let mut pubsub_client = controller.pubsub_client.clone().unwrap();
        let msg = Message::new("Aggregator".to_string(), command.to_abi());
        pubsub_client.send(msg);
    }
    pub fn ownership_property(&self, session: &Bytes, to_address: Address) -> (Property, Metadata) {
        (
            Self::create_ownership_state_object(to_address),
            Metadata::new(self.get_my_address(session).unwrap(), to_address),
        )
    }
    pub fn open_channel_property(
        &self,
        session: &Bytes,
        counter_party_address: Address,
    ) -> (Property, Metadata) {
        let my_address = self.get_my_address(session).unwrap();
        (
            Self::create_channel_state_object(my_address, counter_party_address),
            Metadata::new(my_address, counter_party_address),
        )
    }
    pub fn initialize(&self) {
        let controller = self.controller.clone().unwrap();
        //        controller.fetch_block(Integer(0));
        controller.initialize()
    }
    /// Gets balances as HashMap which key is token address and value is balance
    pub fn get_balance(&self, session: &Bytes) -> HashMap<Address, u64> {
        let controller = self.controller.clone().unwrap();
        let plasma_client = controller.plasma_client.lock().unwrap();
        let my_address = plasma_client.get_my_address(session).unwrap();
        let balances: HashMap<Address, u64> = plasma_client
            .get_all_state_updates()
            .iter()
            .filter(|s| {
                let p = &s.get_property().inputs[2];
                if let PropertyInput::ConstantProperty(signed_by) = p {
                    if let PropertyInput::ConstantAddress(address) = signed_by.inputs[0] {
                        return address == my_address;
                    }
                }
                false
            })
            .fold(HashMap::new(), |mut acc, s| {
                let deposit_contract = s.get_deposit_contract_address();
                let b = acc.get(&deposit_contract).unwrap_or(&0);
                let new_balance = b + s.get_range().get_end() - s.get_range().get_start();
                acc.insert(deposit_contract, new_balance);
                acc
            });
        balances
    }
    pub fn get_related_transactions(&self, session: &Bytes) -> Vec<Transaction> {
        self.controller
            .clone()
            .unwrap()
            .get_related_transactions(session)
    }
    // TODO: get dynamically using token map?
    pub fn get_all_tokens(&self) -> Vec<Token> {
        vec![
            Token::new("ETH", Address::zero()),
            Token::new(
                "DAI",
                string_to_address("0000000000000000000000000000000000000001"),
            ),
        ]
    }
    pub fn get_token_name(&self, address: Address) -> String {
        if address == Address::zero() {
            "ETH".to_string()
        } else if address == string_to_address("0000000000000000000000000000000000000001") {
            "DAI".to_string()
        } else {
            panic!("No token found")
        }
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
    fn search_range(&self, deposit_contract_address: Address, amount: u64) -> Option<Range> {
        self.plasma_client
            .lock()
            .unwrap()
            .search_range(deposit_contract_address, amount)
    }
    fn get_related_transactions(&self, session: &Bytes) -> Vec<Transaction> {
        self.plasma_client
            .lock()
            .unwrap()
            .get_related_transactions(session)
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
    deposit_contract_address: Address,
    decider: PropertyExecutor<KVS>,
}

impl<KVS: KeyValueStore + DatabaseTrait> PlasmaClient<KVS> {
    pub fn new(deposit_contract_address: Address) -> Self {
        PlasmaClient {
            deposit_contract_address,
            decider: Default::default(),
        }
    }

    /// Deposit to plasma contract
    /// Send ethereum transaction to Plasma Deposit Contract.
    /// amount: amount to deposit
    /// property: initial state object
    pub fn deposit(&self, session: &Bytes, amount: u64, property: Property) {
        // TODO: add PlasmaContractABI
        let f = File::open("PlasmaContract.json").unwrap();
        let reader = BufReader::new(f);
        let contract_abi = ContractABI::load(reader).unwrap();
        let plasma_contract = PlasmaContractAdaptor::new(
            "http://127.0.0.1:9545",
            &self.deposit_contract_address.to_string(),
            contract_abi,
        )
        .unwrap();
        // TODO: handle result
        let _result =
            plasma_contract.deposit(self.get_my_address(session).unwrap(), amount, property);
    }

    /// Creates new account
    pub fn create_account(&self) -> (Bytes, SecretKey) {
        let mut wallet = WalletManager::new(self.decider.get_db());
        wallet.generate_key_session()
    }

    pub fn import_key(&self, secret_key: &[u8]) -> (Bytes, SecretKey) {
        let mut wallet = WalletManager::new(self.decider.get_db());
        wallet.import_key(secret_key)
    }

    pub fn get_my_address(&self, session: &Bytes) -> Option<Address> {
        let wallet = WalletManager::new(self.decider.get_db());
        wallet
            .get_key(session)
            .map(|secret_key| secret_key.public().address().into())
    }

    /// Create transaction to update state for specific coin range.
    /// TODO: maybe need to specify Property for how state transition works.
    pub fn create_transaction(
        &self,
        session: &Bytes,
        deposit_contract_address: Address,
        range: Range,
        parameters: Bytes,
        metadata: Metadata,
    ) -> Transaction {
        let transaction_params =
            TransactionParams::new(deposit_contract_address, range, parameters);

        let wallet = WalletManager::new(self.decider.get_db());
        if let Some(secret_key) = wallet.get_key(session) {
            let signature =
                SignVerifier::sign(&secret_key, &Bytes::from(transaction_params.to_abi()));
            Transaction::from_params(transaction_params, signature, metadata)
        } else {
            panic!("secret key not found");
        }
    }

    /// Start exit on plasma. return exit property
    pub fn get_exit_claim(&self, block_number: Integer, range: Range) -> Property {
        // TODO: decide property and claim property to contract
        // TODO: store as exit list
        PlasmaClientShell::create_checkpoint_property(block_number, range)
    }

    /// Handle exit on plasma.
    /// After dispute period, withdraw from Plasma Contract.
    pub fn finalize_exit(&self, session: &Bytes, state_update: StateUpdate, range: Range) {
        // TODO: add PlasmaContractABI
        let f = File::open("PlasmaContract.json").unwrap();
        let reader = BufReader::new(f);
        let contract_abi = ContractABI::load(reader).unwrap();
        let plasma_contract = PlasmaContractAdaptor::new(
            "http://127.0.0.1:9545",
            &self.deposit_contract_address.to_string(),
            contract_abi,
        )
        .unwrap();

        // TODO: create checkpoint struct
        // TODO: decide check point is exitable
        let checkpoint = (state_update, range);

        // TODO: handle result
        let _result = plasma_contract.withdraw(self.get_my_address(session).unwrap(), checkpoint);
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
                    block.get_plasma_data_block(root.clone(), s.clone()),
                )
                .is_ok());
        }
        for tx in block.get_transactions().iter() {
            for previous_block_number in tx.clone().prev_state_block_numbers {
                transaction_db.put_transaction(previous_block_number.0, tx.transaction.clone());
            }
            let message = Bytes::from(tx.transaction.to_body_abi());
            assert!(signed_by_db
                .store_witness(
                    SignVerifier::recover(tx.transaction.get_signature(), &message),
                    message,
                    tx.transaction.get_signature().clone(),
                )
                .is_ok());
        }
        for su in self.get_all_state_updates() {
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
        let _ = self.decider.get_db().put(
            &Bytes::from(&b"latest_block_number"[..]).into(),
            &Bytes::from(Integer::new(block.get_block_number())),
        );
    }

    fn get_latest_block_number(&self) -> u64 {
        let result = self
            .decider
            .get_db()
            .get(&Bytes::from(&b"latest_block_number"[..]).into())
            .unwrap()
            .unwrap();
        Integer::from(Bytes::from(result)).0
    }

    pub fn handle_new_transaction(&self, event: &NewTransactionEvent) {
        println!("handle_new_transaction");
        // println!("handle_new_transaction {:?}", event);
        let transaction_db = TransactionDb::new(self.decider.get_range_db());
        for previous_block_number in event.clone().prev_state_block_numbers {
            transaction_db.put_transaction(previous_block_number.0, event.transaction.clone());
        }
    }

    pub fn insert_test_ranges(&mut self) {
        let mut state_updates = vec![];
        let eth_token_address = Address::zero();
        let dai_token_address = string_to_address("0000000000000000000000000000000000000001");
        for i in 0..3 {
            state_updates.push(StateUpdate::new(
                Integer::new(0),
                eth_token_address,
                Range::new(i * 20, (i + 1) * 20),
                PlasmaClientShell::create_ownership_state_object(string_to_address(
                    "627306090abab3a6e1400e9345bc60c78a8bef57",
                )),
            ));
        }
        for i in 0..3 {
            state_updates.push(StateUpdate::new(
                Integer::new(0),
                dai_token_address,
                Range::new(i * 100, (i + 1) * 100),
                PlasmaClientShell::create_ownership_state_object(string_to_address(
                    "627306090abab3a6e1400e9345bc60c78a8bef57",
                )),
            ));
        }

        let plasma_block = PlasmaBlock::new(0, state_updates, vec![]);
        self.handle_new_block(plasma_block);
    }

    pub fn get_all_state_updates(&self) -> Vec<StateUpdate> {
        let range_db = self.decider.get_range_db();
        let state_db = StateDb::new(range_db);
        state_db.get_all_state_updates().unwrap_or_else(|_| vec![])
    }

    pub fn get_state_updates(&self, deposit_contract_address: Address) -> Vec<StateUpdate> {
        let range_db = self.decider.get_range_db();
        let state_db = StateDb::new(range_db);
        state_db
            .get_verified_state_updates(deposit_contract_address, 0, std::u64::MAX)
            .unwrap_or_else(|_| vec![])
    }

    pub fn update_state_updates(&self, state_updates: Vec<StateUpdate>) {
        let range_db = self.decider.get_range_db();
        let mut state_db = StateDb::new(range_db);

        for s in state_updates.iter() {
            let _ = state_db.put_verified_state_update(&s);
        }
    }

    /// return range if enough amount is exists.
    pub fn search_range(&self, deposit_contract_address: Address, amount: u64) -> Option<Range> {
        // TODO: decide if this property is owner's property.
        self.get_state_updates(deposit_contract_address)
            .iter()
            .map(|su| su.get_range())
            .find(|range| amount <= range.get_end() - range.get_start())
    }

    fn get_related_transactions(&self, session: &Bytes) -> Vec<Transaction> {
        let address = self.get_my_address(session).unwrap();
        println!("{:?}", address);
        let transaction_db = TransactionDb::new(self.decider.get_range_db());
        let latest_block_number = self.get_latest_block_number();

        let filter = TransactionFilterBuilder::new()
            .address_to(address)
            .address_from(address)
            .block_from(0)
            .block_to(latest_block_number)
            .range(Range::new(0, 1000)) // TODO: max range?
            .build();

        transaction_db.query_transaction(filter).unwrap()
    }
}
