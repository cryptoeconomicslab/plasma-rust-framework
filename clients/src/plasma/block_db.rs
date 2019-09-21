use super::command::NewTransactionEvent;
use super::error::{Error, ErrorKind};
use super::plasma_block::PlasmaBlock;
use abi_utils::{Decodable, Encodable};
use bytes::Bytes;
use ovm::types::{Integer, StateUpdate};
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

        self.db
            .bucket(&Bytes::from(&"queued_state_updates"[..]))
            .put(range.get_start(), range.get_end(), &state_update.to_abi())
            .map_err::<Error, _>(Into::into)?;
        Ok(())
    }

    pub fn get_queued_state_updates(
        &self,
        start: u64,
        end: u64,
    ) -> Result<Vec<StateUpdate>, Error> {
        let res = self
            .db
            .bucket(&Bytes::from(&"queued_state_updates"[..]))
            .get(start, end)?
            .iter()
            .map(|range| StateUpdate::from_abi(range.get_value()).unwrap())
            .collect();

        Ok(res)
    }

    pub fn get_pending_state_updates(&self) -> Result<Vec<StateUpdate>, Error> {
        let res = self
            .db
            .bucket(&Bytes::from(&"queued_state_updates"[..]))
            .get(MIN_RANGE, MAX_RANGE)?
            .iter()
            .map(|range| StateUpdate::from_abi(range.get_value()).unwrap())
            .collect();
        Ok(res)
    }

    pub fn delete_all_queued_state_updates(&self) -> Result<(), Error> {
        let _ = self
            .db
            .bucket(&Bytes::from(&"queued_state_updates"[..]))
            .del_batch(MIN_RANGE, MAX_RANGE)?;
        Ok(())
    }

    pub fn enqueue_tx(&self, tx: NewTransactionEvent) -> Result<(), Error> {
        let range = tx.transaction.get_range();

        self.db
            .bucket(&Bytes::from(&"queued_txs"[..]))
            .put(range.get_start(), range.get_end(), &tx.to_abi())
            .map_err::<Error, _>(Into::into)?;
        Ok(())
    }

    pub fn get_pending_txs(&self) -> Result<Vec<NewTransactionEvent>, Error> {
        let res = self
            .db
            .bucket(&Bytes::from(&"queued_txs"[..]))
            .get(MIN_RANGE, MAX_RANGE)?
            .iter()
            .map(|range| NewTransactionEvent::from_abi(range.get_value()).unwrap())
            .collect();
        Ok(res)
    }

    pub fn delete_all_queued_txs(&self) -> Result<(), Error> {
        let _ = self
            .db
            .bucket(&Bytes::from(&"queued_txs"[..]))
            .del_batch(MIN_RANGE, MAX_RANGE)?;
        Ok(())
    }

    pub fn get_block(&self, block_number: Integer) -> Result<PlasmaBlock, Error> {
        let plasma_block_opt = self
            .db
            .get_db()
            .bucket(&Bytes::from("plasma_block_db").into())
            .bucket(&Bytes::from("blocks").into())
            .get(&block_number.0.into())
            .map_err::<Error, _>(Into::into)?;
        if let Some(plasma_block) = plasma_block_opt {
            PlasmaBlock::from_abi(&plasma_block).map_err(Into::into)
        } else {
            Err(Error::from(ErrorKind::PlasmaDbError))
        }
    }

    pub fn save_block(&self, block: &PlasmaBlock) -> Result<(), Error> {
        let index = block.get_block_number();
        self.db
            .get_db()
            .bucket(&Bytes::from("plasma_block_db").into())
            .bucket(&Bytes::from("blocks").into())
            .put(&index.into(), &block.to_abi())?;
        Ok(())
    }
}
