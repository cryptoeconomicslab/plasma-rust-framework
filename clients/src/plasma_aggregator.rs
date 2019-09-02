use super::error::{Error, ErrorKind};
use super::state_db::StateDb;
use ethereum_types::Address;
use ethsign::SecretKey;
use ovm::db::SignedByDb;
use ovm::property_executor::PropertyExecutor;
use ovm::types::PlasmaDataBlock;
use plasma_core::data_structure::{PlasmaBlock, Transaction};
use plasma_db::impls::kvs::memory::CoreDbMemoryImpl;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::RangeDbImpl;

pub struct PlasmaAggregator<'a, KVS: KeyValueStore> {
    plasma_contract_address: Address,
    state_db: StateDb<'a, KVS>,
    secret_key: SecretKey,
    my_address: Address,
    //    state_update_queue:
}

impl<'a, KVS: KeyValueStore + DatabaseTrait> PlasmaAggregator<'a, KVS> {
    pub fn new(
        plasma_contract_address: Address,
        private_key: &str,
        range_db: &'a RangeDbImpl<KVS>,
    ) -> Self {
        let raw_key = hex::decode(private_key).unwrap();
        let secret_key = SecretKey::from_raw(&raw_key).unwrap();
        let my_address: Address = secret_key.public().address().into();
        let state_db = StateDb::from(&range_db);

        PlasmaAggregator {
            plasma_contract_address,
            secret_key,
            my_address,
            state_db,
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
        let plasma_data_blocks = self
            .state_db
            .get_verified_plasma_data_blocks(
                transaction.get_range().get_start(),
                transaction.get_range().get_end(),
            )
            .unwrap();
        let prev_state = &plasma_data_blocks[0];
        if !prev_state
            .get_updated_range()
            .is_subrange(&transaction.get_range())
        {
            return Err(Error::from(ErrorKind::InvalidTransaction));
        }
        let next_state = prev_state.transition(&transaction);
        // verify state transition
        if !prev_state.verify_deprecation(&transaction) {
            return Err(Error::from(ErrorKind::InvalidTransaction));
        }
        self.enqueue_state_update(next_state);
        Ok(())
    }

    fn enqueue_state_update(&mut self, state_update: PlasmaDataBlock) {
        // TODO: implement
        // store in range db
    }

    fn generate_block(&self) -> PlasmaBlock {
        // dequeue all plasma_data_block stored in range db
        // generate block using that data.
        PlasmaBlock {}
    }
}
