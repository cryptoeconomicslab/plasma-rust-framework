use super::types::BlockNumber;
use crate::error::Error;
use bytes::{BigEndian, ByteOrder};
use plasma_core::data_structure::StateUpdate;
use plasma_db::impls::kvs::kvdb::CoreDb;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::traits::rangestore::RangeStore;

pub struct BlockDb {
    db: Box<CoreDb>,
}

impl Default for BlockDb {
    fn default() -> Self {
        Self {
            db: Box::new(CoreDb::open("test")),
        }
    }
}

impl BlockDb {
    pub fn add_pending_state_update(&self, _state_update: StateUpdate) {}
    pub fn get_state_updates(&self) -> [StateUpdate] {
        []
    }
    pub fn get_next_block_number(&self) -> Result<BlockNumber, Error> {
        let next_block = self
            .db
            .get(&b"next_block"[..])
            .map_err::<Error, _>(Into::into)?;
        Ok(BigEndian::read_u64(&next_block.unwrap()))
    }
    pub fn get_next_block_store(&self) {}
}
