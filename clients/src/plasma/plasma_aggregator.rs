use super::block_manager::BlockManager;
use super::command::NewTransactionEvent;
use super::error::{Error, ErrorKind};
use super::plasma_block::PlasmaBlock;
use super::plasma_client::PlasmaClientShell;
use super::state_db::StateDb;
use bytes::Bytes;
use ethereum_types::Address;
use ethsign::SecretKey;
use ovm::db::{SignedByDb, TransactionDb};
use ovm::deciders::SignVerifier;
use ovm::property_executor::PropertyExecutor;
use ovm::types::Integer;
use ovm::types::{StateUpdate, StateUpdateList};
use plasma_core::data_structure::{Range, Transaction};
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;

pub struct PlasmaAggregator<KVS: KeyValueStore> {
    aggregator_address: Address,
    commitment_contract_address: Address,
    plasma_contract_address: Address,
    //_secret_key: SecretKey,
    _raw_key: Vec<u8>,
    _my_address: Address,
    block_manager: BlockManager<KVS>,
    decider: PropertyExecutor<KVS>,
    //_secret_key: SecretKey,
    //    state_update_queue:
}

impl<KVS: KeyValueStore + DatabaseTrait> PlasmaAggregator<KVS> {
    pub fn new(
        aggregator_address: Address,
        plasma_contract_address: Address,
        commitment_contract_address: Address,
        private_key: &str,
    ) -> Self {
        let raw_key = hex::decode(private_key).unwrap();
        let secret_key = SecretKey::from_raw(&raw_key).unwrap();
        let my_address: Address = secret_key.public().address().into();
        let block_manager = BlockManager::new(aggregator_address, commitment_contract_address);

        PlasmaAggregator {
            aggregator_address,
            plasma_contract_address,
            commitment_contract_address,
            _raw_key: raw_key,
            //_secret_key: secret_key,
            _my_address: my_address,
            block_manager,
            decider: Default::default(),
        }
    }

    // 1. query all state_updates overlapping with given range.
    // 2. check if the range of transaction is covered by queried state_updates. If not, return
    //    InvalidTransaction Error.
    // 3. for all state_updates, check state transition using state_update.property.decide(transaction).
    //    any of these throw error, return InvalidTransaction Error.
    // 4. if all transitions are verified, add new state_update to a queue.
    //
    // TODO:
    // - handle multi prev_states case.
    // - fix decide logic for state transition.
    pub fn ingest_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<NewTransactionEvent, Error> {
        let transaction_db = TransactionDb::new(self.decider.get_range_db());
        let signed_by_db = SignedByDb::new(self.decider.get_db());
        let next_block_number = self.block_manager.get_current_block_number();
        let state_db = StateDb::new(self.decider.get_range_db());
        let state_updates = state_db
            .get_verified_state_updates(
                transaction.get_range().get_start(),
                transaction.get_range().get_end(),
            )
            .unwrap();
        if state_updates.is_empty() {
            return Err(Error::from(ErrorKind::InvalidTransaction));
        }
        let prev_state = &state_updates[0];
        transaction_db.put_transaction(prev_state.get_block_number().0, transaction.clone());
        let message = Bytes::from(transaction.to_body_abi());
        assert!(signed_by_db
            .store_witness(
                SignVerifier::recover(transaction.get_signature(), &message),
                message,
                transaction.get_signature().clone(),
            )
            .is_ok());

        if !prev_state.get_range().is_subrange(&transaction.get_range()) {
            return Err(Error::from(ErrorKind::InvalidTransaction));
        }
        if let Ok(next_state) = prev_state.execute_state_transition(
            &mut self.decider,
            &transaction,
            Integer(next_block_number),
        ) {
            let res = self.block_manager.enqueue_state_update(next_state);
            if res.is_err() {
                return Err(Error::from(ErrorKind::InvalidTransaction));
            }
            let new_tx =
                NewTransactionEvent::new(prev_state.get_block_number(), transaction.clone());
            let res_tx = self.block_manager.enqueue_tx(new_tx.clone());
            if res_tx.is_err() {
                return Err(Error::from(ErrorKind::InvalidTransaction));
            }
            return Ok(new_tx.clone());
        }
        Err(Error::from(ErrorKind::InvalidTransaction))
    }

    pub fn submit_next_block(&mut self) -> Result<(), Error> {
        // dequeue all state_update stored in range db
        // generate block using that data.
        let block_manager = &mut self.block_manager;
        block_manager.submit_next_block()
    }

    pub fn get_aggregator_addres(&self) -> Address {
        self.aggregator_address
    }

    pub fn get_commitment_contract_address(&self) -> Address {
        self.commitment_contract_address
    }

    pub fn get_plasma_contract_address(&self) -> Address {
        self.plasma_contract_address
    }

    pub fn show_queued_state_updates(&self) {
        println!("{:?}", self.block_manager.get_queued_state_updates());
    }

    pub fn insert_test_ranges(&mut self) {
        let mut state_db = StateDb::new(self.decider.get_range_db());
        let eth_token_address = Address::zero();
        for i in 0..3 {
            let state_update = StateUpdate::new(
                Integer::new(0),
                eth_token_address,
                Range::new(i * 20, (i + 1) * 20),
                PlasmaClientShell::create_ownership_state_object(Address::from_slice(
                    &hex::decode("627306090abab3a6e1400e9345bc60c78a8bef57").unwrap(),
                )),
            );
            assert!(state_db.put_verified_state_update(state_update).is_ok());
        }
    }

    pub fn get_all_state_updates(&self) -> Vec<StateUpdate> {
        let state_db = StateDb::new(self.decider.get_range_db());
        state_db.get_all_state_updates().unwrap_or_else(|_| vec![])
    }

    pub fn get_state_updates_of_block(
        &self,
        block_number: Integer,
    ) -> Result<StateUpdateList, Error> {
        self.block_manager
            .get_block_range(block_number)
            .map(|b| StateUpdateList::new(b.get_state_updates().to_vec()))
    }

    pub fn get_plasma_block_of_block(&self, block_number: Integer) -> Result<PlasmaBlock, Error> {
        self.block_manager.get_block_range(block_number)
    }
}
