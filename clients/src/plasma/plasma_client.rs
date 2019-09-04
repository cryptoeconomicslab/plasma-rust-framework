use bytes::Bytes;
use contract_wrapper::plasma_contract_adaptor::PlasmaContractAdaptor;
use ethabi::Contract as ContractABI;
use ethereum_types::Address;
use ethsign::SecretKey;
use ovm::deciders::SignVerifier;
use ovm::statements::create_plasma_property;
use ovm::types::core::Property;
use ovm::types::Integer;
use plasma_core::data_structure::abi::Encodable;
use plasma_core::data_structure::{
    PlasmaBlock, Range, StateUpdate, Transaction, TransactionParams,
};
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use std::fs::File;
use std::io::BufReader;

/// Plasma Client on OVM.
pub struct PlasmaClient<KVS> {
    plasma_contract_address: Address,
    db: KVS,
    secret_key: SecretKey,
    my_address: Address,
}

impl<KVS: KeyValueStore + DatabaseTrait> PlasmaClient<KVS> {
    pub fn new(plasma_contract_address: Address, private_key: &str) -> Self {
        let raw_key = hex::decode(private_key).unwrap();
        let secret_key = SecretKey::from_raw(&raw_key).unwrap();
        let my_address: Address = secret_key.public().address().into();

        PlasmaClient {
            plasma_contract_address,
            db: KVS::open("plasma_db"),
            secret_key,
            my_address,
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
            &self.plasma_contract_address.clone().to_string(),
            contract_abi,
        )
        .unwrap();
        // TODO: handle result
        let result = plasma_contract.deposit(self.my_address, amount, property);
    }

    /// Create transaction to update state for specific coin range.
    /// TODO: maybe need to specify Property for how state transition works.
    pub fn create_transaction(&self, range: Range, parameters: Bytes) -> Transaction {
        let transaction_params =
            TransactionParams::new(self.plasma_contract_address, range, parameters);

        let signature =
            SignVerifier::sign(&self.secret_key, &Bytes::from(transaction_params.to_abi()));
        let signed_tx = Transaction::from_params(transaction_params, signature);
        signed_tx
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
            &self.plasma_contract_address.clone().to_string(),
            contract_abi,
        )
        .unwrap();

        // TODO: create checkpoint struct
        // TODO: decide check point is exitable
        let checkpoint = (state_update, range);

        // TODO: handle result
        let result = plasma_contract.withdraw(self.my_address, checkpoint);
    }

    /// Challenge to specific exit by claiming contradicting statement.
    pub fn challenge(&self) {}

    /// Handle BlockSubmitted Event from aggregator
    /// check new state update and verify, store them.
    pub fn handle_new_block(&self, block: PlasmaBlock) {

    }
}
