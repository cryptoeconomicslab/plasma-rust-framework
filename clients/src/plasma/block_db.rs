use super::plasma_block::PlasmaBlock;
use super::state_update::StateUpdate;
use bytes::Bytes;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_db::error::Error;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::traits::rangestore::RangeStore;
use plasma_db::RangeDbImpl;

const MIN_RANGE: u64 = 0;
const MAX_RANGE: u64 = std::u64::MAX;

pub struct BlockDb<'a, KVS: KeyValueStore> {
    db: &'a RangeDbImpl<KVS>,
}

impl<'a, KVS: KeyValueStore> BlockDb<'a, KVS> {
    pub fn from(range_db: &'a RangeDbImpl<KVS>) -> Self {
        BlockDb { db: range_db }
    }

    pub fn enqueue_state_update(&self, state_update: StateUpdate) -> Result<(), Error> {
        let range = state_update.get_range();

        let _ = self.db.bucket(&Bytes::from(&"plasma_block_db"[..])).put(
            range.get_start(),
            range.get_end(),
            &state_update.to_abi(),
        )?;
        Ok(())
    }

    pub fn get_queued_state_updates(
        &self,
        start: u64,
        end: u64,
    ) -> Result<Vec<StateUpdate>, Error> {
        let res = self
            .db
            .bucket(&Bytes::from(&"plasma_block_db"[..]))
            .get(start, end)?
            .iter()
            .map(|range| StateUpdate::from_abi(range.get_value()).unwrap())
            .collect();

        Ok(res)
    }

    pub fn get_pending_state_updates(&self) -> Result<Vec<StateUpdate>, Error> {
        let res = self
            .db
            .bucket(&Bytes::from(&"plasma_block_db"[..]))
            .get(MIN_RANGE, MAX_RANGE)?
            .iter()
            .map(|range| StateUpdate::from_abi(range.get_value()).unwrap())
            .collect();
        Ok(res)
    }

    pub fn delete_all_queued_state_updates(&self) -> Result<(), Error> {
        let _ = self
            .db
            .bucket(&Bytes::from(&"plasma_block_db"[..]))
            .del_batch(MIN_RANGE, MAX_RANGE)?;
        Ok(())
    }

    pub fn save_block(&self, block: &PlasmaBlock) -> Result<(), Error> {
        let index = block.get_block_number();
        let _ = self
            .db
            .bucket(&Bytes::from(&"plasma_block_db"[..]))
            .bucket(&Bytes::from(&"blocks"[..]))
            .put(index, index, &block.to_abi())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BlockDb;
    use bytes::Bytes;
    use ethereum_types::Address;
    use ovm::types::{Integer, Property, SignedByInput, StateUpdate};
    use plasma_core::data_structure::Range;
    use plasma_db::impls::kvs::memory::CoreDbMemoryImpl;
    use plasma_db::traits::db::DatabaseTrait;
    use plasma_db::RangeDbImpl;

    #[test]
    fn test_enqueue() {
        let base_db = CoreDbMemoryImpl::open("test");
        let db = RangeDbImpl::from(base_db);
        let mut block_db = BlockDb::from(&db);
        let address: Address = Address::zero();

        let state_update = StateUpdate::new(
            Integer::new(1),
            Range::new(0, 100),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
            Bytes::from(&b"root"[..]),
        );

        let _ = block_db.enqueue_state_update(state_update);
        let result = block_db.get_queued_state_updates(0, 100).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_get_queued_state_updates() {
        let base_db = CoreDbMemoryImpl::open("test");
        let db = RangeDbImpl::from(base_db);
        let mut block_db = BlockDb::from(&db);
        let address: Address = Address::zero();

        let state_update = StateUpdate::new(
            Integer::new(1),
            Range::new(0, 10),
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
        );

        let state_update2 = StateUpdate::new(
            Integer::new(1),
            Range::new(10, 100),
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
        );

        let state_update3 = StateUpdate::new(
            Integer::new(1),
            Range::new(100, 115),
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
        );

        let _ = block_db.enqueue_state_update(state_update);
        let _ = block_db.enqueue_state_update(state_update2);
        let _ = block_db.enqueue_state_update(state_update3);
        let result = block_db.get_queued_state_updates(0, 100).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_get_pending_state_updates() {
        let base_db = CoreDbMemoryImpl::open("test");
        let db = RangeDbImpl::from(base_db);
        let mut block_db = BlockDb::from(&db);
        let address: Address = Address::zero();

        let state_update = StateUpdate::new(
            Integer::new(1),
            Range::new(0, 10),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
            Bytes::from(&b"root"[..]),
        );

        let state_update2 = StateUpdate::new(
            Integer::new(1),
            Range::new(10, 100),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
            Bytes::from(&b"root"[..]),
        );

        let state_update3 = StateUpdate::new(
            Integer::new(1),
            Range::new(100, 115),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
            Bytes::from(&b"root"[..]),
        );

        let _ = block_db.enqueue_state_update(state_update);
        let _ = block_db.enqueue_state_update(state_update2);
        let _ = block_db.enqueue_state_update(state_update3);
        let result = block_db.get_pending_state_updates().unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_delete_all_queued_state_updates() {
        let base_db = CoreDbMemoryImpl::open("test");
        let db = RangeDbImpl::from(base_db);
        let mut block_db = BlockDb::from(&db);
        let address: Address = Address::zero();

        let state_update = StateUpdate::new(
            Integer::new(1),
            Range::new(0, 10),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
            Bytes::from(&b"root"[..]),
        );

        let state_update2 = StateUpdate::new(
            Integer::new(1),
            Range::new(10, 100),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
            Bytes::from(&b"root"[..]),
        );

        let state_update3 = StateUpdate::new(
            Integer::new(1),
            Range::new(100, 115),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
            Bytes::from(&b"root"[..]),
        );

        let _ = block_db.enqueue_state_update(state_update);
        let _ = block_db.enqueue_state_update(state_update2);
        let _ = block_db.enqueue_state_update(state_update3);
        let _ = block_db.delete_all_queued_state_updates().unwrap();
        let result = block_db.get_pending_state_updates().unwrap();
        assert_eq!(result.len(), 0);
    }

}
