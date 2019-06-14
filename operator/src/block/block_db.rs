use super::types::BlockNumber;
use crate::error::Error;
use bytes::Bytes;
use plasma_core::data_structure::StateUpdate;
use plasma_db::impls::rangedb::RangeDbImpl;
use plasma_db::range::Range;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::{BaseDbKey, Bucket, KeyValueStore};
use plasma_db::traits::rangestore::RangeStore;

static NEXT_BLOCK_KEY: &[u8; 10] = b"next_block";

/// Database to store blocks
pub struct BlockDb<D> {
    var_db: Box<D>,
    db: Box<D>,
}

impl<D> Default for BlockDb<D>
where
    D: DatabaseTrait + KeyValueStore<Range>,
{
    fn default() -> Self {
        Self {
            var_db: Box::new(D::open("var")),
            db: Box::new(D::open("blocks")),
        }
    }
}

impl<D> BlockDb<D>
where
    D: DatabaseTrait + KeyValueStore<Range>,
{
    pub fn set_block_number(&self, block_number: BlockNumber) -> Result<(), Error> {
        let value: Bytes = block_number.into();
        self.var_db
            .put(&BaseDbKey::from(&NEXT_BLOCK_KEY[..]), &value[..].to_vec())
            .map_err::<Error, _>(Into::into)
    }
    /// Adds new StateUpdate
    pub fn add_pending_state_update(&self, state_update: StateUpdate) -> Result<(), Error> {
        let block_number = self.get_next_block_number()?;
        let rangedb = self.get_block_store(block_number);
        let start = state_update.get_start();
        let end = state_update.get_end();
        let value = state_update.to_abi();
        rangedb
            .put(start, end, &value)
            .map_err::<Error, _>(Into::into)
    }
    // Gets all pending StateUpdates
    pub fn get_pending_state_updates(&self) -> Result<Vec<StateUpdate>, Error> {
        let block_number = self.get_next_block_number()?;
        self.get_state_updates(block_number)
    }
    pub fn get_state_updates(&self, block_number: BlockNumber) -> Result<Vec<StateUpdate>, Error> {
        let rangedb = self.get_block_store(block_number);
        let ranges = rangedb
            .get(0, 0xffff_ffff_ffff_ffff)
            .map_err::<Error, _>(Into::into)?;
        Ok(ranges
            .iter()
            .map(|range| StateUpdate::from_abi(range.get_value()).ok().unwrap())
            .collect())
    }
    pub fn get_next_block_number(&self) -> Result<BlockNumber, Error> {
        let next_block = self
            .var_db
            .get(&BaseDbKey::from(&NEXT_BLOCK_KEY[..]))
            .map_err::<Error, _>(Into::into)?;
        Ok(BlockNumber::from(Bytes::from(
            next_block.unwrap().as_slice(),
        )))
    }
    pub fn get_next_block_store(&self) -> Result<RangeDbImpl<Bucket<Range>>, Error> {
        let next_block_number = self.get_next_block_number()?;
        Ok(self.get_block_store(next_block_number))
    }
    pub fn get_block_store(&self, block_number: BlockNumber) -> RangeDbImpl<Bucket<Range>> {
        let key: BaseDbKey = block_number.into();
        let bucket = self.db.bucket(&key);
        RangeDbImpl::from(bucket)
    }
    /// Finalize current block
    pub fn finalize_block(&self) -> Result<(), Error> {
        let next_block_number = self.get_next_block_number()?;
        self.set_block_number(next_block_number + BlockNumber::new(1))
    }
}

#[cfg(test)]
mod tests {
    use super::BlockDb;
    use super::BlockNumber;
    use ethereum_types::Address;
    use plasma_core::data_structure::{StateObject, StateUpdate};
    use plasma_db::impls::kvs::CoreDbMemoryImpl;

    #[test]
    fn test_get_pending_state_updates() {
        let data = Vec::from(&b"data"[..]);
        let state_object = StateObject::new(Address::zero(), &data);
        let state_update = StateUpdate::new(state_object, 0, 100, 1, Address::zero());

        let block_db: BlockDb<CoreDbMemoryImpl> = Default::default();
        assert!(block_db.set_block_number(BlockNumber::new(0)).is_ok());
        assert!(block_db.add_pending_state_update(state_update).is_ok());
        let state_updates = block_db.get_pending_state_updates().ok().unwrap();
        assert_eq!(state_updates.len(), 1);
    }

    #[test]
    fn test_finalize_block() {
        let data = Vec::from(&b"data"[..]);
        let state_object = StateObject::new(Address::zero(), &data);
        let state_update = StateUpdate::new(state_object, 0, 100, 1, Address::zero());

        let block_db: BlockDb<CoreDbMemoryImpl> = Default::default();
        assert!(block_db.set_block_number(BlockNumber::new(0)).is_ok());
        assert!(block_db.add_pending_state_update(state_update).is_ok());
        assert!(block_db.finalize_block().is_ok());
        let state_updates = block_db.get_pending_state_updates().ok().unwrap();
        assert_eq!(state_updates.len(), 0);
    }

}
