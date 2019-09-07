use super::plasma_block::PlasmaBlock;
use bytes::Bytes;
use ovm::types::StateUpdate;
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

        self.db.bucket(&Bytes::from(&"plasma_block_db"[..])).put(
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
        self
            .db
            .bucket(&Bytes::from(&"plasma_block_db"[..]))
            .bucket(&Bytes::from(&"blocks"[..]))
            .put(index, index, &block.to_abi())?;
        Ok(())
    }
}
