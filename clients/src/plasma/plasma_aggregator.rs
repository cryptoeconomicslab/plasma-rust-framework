use super::block_manager::BlockManager;
use super::error::{Error, ErrorKind};
use super::state_db::StateDb;
use ethereum_types::Address;
use ethsign::SecretKey;
use ovm::db::TransactionDb;
use plasma_core::data_structure::Transaction;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::RangeDbImpl;

pub struct PlasmaAggregator<'a, KVS: KeyValueStore> {
    aggregator_address: Address,
    commitment_contract_address: Address,
    plasma_contract_address: Address,
    state_db: StateDb<'a, KVS>,
    _secret_key: SecretKey,
    _my_address: Address,
    block_manager: BlockManager<KVS>,
    db: KVS,
    //    state_update_queue:
}

impl<'a, KVS: KeyValueStore + DatabaseTrait + Copy> PlasmaAggregator<'a, KVS> {
    pub fn new(
        aggregator_address: Address,
        plasma_contract_address: Address,
        commitment_contract_address: Address,
        private_key: &str,
        range_db: &'a RangeDbImpl<KVS>,
    ) -> Self {
        let raw_key = hex::decode(private_key).unwrap();
        let secret_key = SecretKey::from_raw(&raw_key).unwrap();
        let my_address: Address = secret_key.public().address().into();
        let state_db = StateDb::from(&range_db);
        let block_manager = BlockManager::new(
            aggregator_address.clone(),
            commitment_contract_address.clone(),
        );

        PlasmaAggregator {
            aggregator_address,
            plasma_contract_address,
            commitment_contract_address,
            _secret_key: secret_key,
            _my_address: my_address,
            state_db,
            block_manager,
            db: KVS::open("aggregator_db"),
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
    pub fn ingest_transaction(&mut self, transaction: Transaction) -> Result<(), Error> {
        let state_updates = self
            .state_db
            .get_verified_state_updates(
                transaction.get_range().get_start(),
                transaction.get_range().get_end(),
            )
            .unwrap();
        let prev_state = &state_updates[0];
        if !prev_state.get_range().is_subrange(&transaction.get_range()) {
            return Err(Error::from(ErrorKind::InvalidTransaction));
        }
        if let Ok(next_state) = prev_state.execute_state_transition(&transaction) {
            let res = self.block_manager.enqueue_state_update(next_state);
            if res.is_err() {
                return Err(Error::from(ErrorKind::InvalidTransaction));
            }

            let range_db = RangeDbImpl::from(self.db);
            let transaction_db = TransactionDb::new(&range_db);
            transaction_db.put_transaction(self.block_manager.get_next_block_number(), transaction);

            return Ok(());
        }
        Err(Error::from(ErrorKind::InvalidTransaction))
    }

    pub fn submit_next_block(&self) {
        // dequeue all state_update stored in range db
        // generate block using that data.
        self.block_manager.submit_next_block();
    }

    pub fn get_aggregator_addres(&self) -> Address {
        self.aggregator_address.clone()
    }

    pub fn get_commitment_contract_address(&self) -> Address {
        self.commitment_contract_address.clone()
    }

    pub fn get_plasma_contract_address(&self) -> Address {
        self.plasma_contract_address.clone()
    }
}
