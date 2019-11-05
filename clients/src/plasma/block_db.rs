use super::command::NewTransactionEvent;
use super::error::{Error, ErrorKind};
use super::plasma_block::PlasmaBlock;
use super::utils::string_to_address;
use abi_utils::{Decodable, Encodable};
use bytes::Bytes;
use ethereum_types::Address;
use ovm::types::{Integer, StateUpdate};
use plasma_db::{
    traits::{kvs::KeyValueStore, rangestore::RangeStore},
    RangeDbImpl,
};
use std::sync::RwLock;

const MIN_RANGE: u64 = 0;
const MAX_RANGE: u64 = std::u64::MAX;

pub struct BlockDb<'a, KVS: KeyValueStore> {
    db: &'a RangeDbImpl<KVS>,
    tx_queue: RwLock<Vec<NewTransactionEvent>>,
}

impl<'a, KVS: KeyValueStore> BlockDb<'a, KVS> {
    pub fn from(range_db: &'a RangeDbImpl<KVS>) -> Self {
        BlockDb {
            db: range_db,
            tx_queue: RwLock::new(vec![]),
        }
    }

    pub fn enqueue_state_update(&self, state_update: &StateUpdate) -> Result<(), Error> {
        let range = state_update.get_range();

        self.db
            .bucket(&Bytes::from(&"queued_state_updates"[..]))
            .bucket(&Bytes::from(
                state_update.get_deposit_contract_address().as_bytes(),
            ))
            .put(range.get_start(), range.get_end(), &state_update.to_abi())
            .map_err::<Error, _>(Into::into)?;
        Ok(())
    }

    pub fn get_pending_state_updates_by_token(
        &self,
        token_address: Address,
    ) -> Result<Vec<StateUpdate>, Error> {
        let res = self
            .db
            .bucket(&Bytes::from(&"queued_state_updates"[..]))
            .bucket(&Bytes::from(token_address.as_bytes()))
            .get(MIN_RANGE, MAX_RANGE)?
            .iter()
            .map(|range| StateUpdate::from_abi(range.get_value()).unwrap())
            .collect();
        Ok(res)
    }

    pub fn get_pending_state_updates(&self) -> Result<Vec<StateUpdate>, Error> {
        let mut result = vec![];
        let mut state_updates = self.get_pending_state_updates_by_token(Address::zero())?;
        result.append(&mut state_updates);
        let mut state_updates = self.get_pending_state_updates_by_token(string_to_address(
            "0000000000000000000000000000000000000001",
        ))?;
        result.append(&mut state_updates);
        Ok(result)
    }

    fn delete_all_queued_state_updates_by_address(
        &self,
        token_address: Address,
    ) -> Result<(), Error> {
        let _ = self
            .db
            .bucket(&Bytes::from(&"queued_state_updates"[..]))
            .bucket(&Bytes::from(token_address.as_bytes()))
            .del_batch(MIN_RANGE, MAX_RANGE)?;
        Ok(())
    }

    pub fn delete_all_queued_state_updates(&self) -> Result<(), Error> {
        self.delete_all_queued_state_updates_by_address(Address::zero())?;
        self.delete_all_queued_state_updates_by_address(string_to_address(
            "0000000000000000000000000000000000000001",
        ))?;
        Ok(())
    }

    pub fn enqueue_tx(&self, tx: NewTransactionEvent) -> Result<(), Error> {
        if let Ok(mut tx_queue) = self.tx_queue.write() {
            tx_queue.push(tx);
        }
        Ok(())
    }

    pub fn get_pending_txs(&self) -> Result<Vec<NewTransactionEvent>, Error> {
        if let Ok(tx_queue) = self.tx_queue.read() {
            Ok(tx_queue.to_vec())
        } else {
            Err(Error::from(ErrorKind::PlasmaDbError))
        }
    }

    pub fn delete_all_queued_txs(&self) -> Result<(), Error> {
        if let Ok(mut tx_queue) = self.tx_queue.write() {
            tx_queue.clear();
            Ok(())
        } else {
            Err(Error::from(ErrorKind::PlasmaDbError))
        }
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

#[cfg(test)]
mod tests {
    use super::super::{command::NewTransactionEvent, plasma_block::PlasmaBlock};
    use super::*;
    use ethereum_types::Address;
    use ovm::types::{Integer, Property, StateUpdate};
    use plasma_core::data_structure::{Metadata, Range, Transaction};
    use plasma_db::{impls::kvs::CoreDbMemoryImpl, traits::DatabaseTrait, RangeDbImpl};

    #[test]
    fn test_save_and_load_block() {
        let db = CoreDbMemoryImpl::open("test");
        let range_db = RangeDbImpl::from(db);
        let block_db = BlockDb::from(&range_db);

        let plasma_block = PlasmaBlock::new(
            1,
            vec![StateUpdate::new(
                Integer::new(1),
                Address::zero(),
                Range::new(0, 5),
                Property::new(Address::zero(), vec![]),
            )],
            vec![NewTransactionEvent::new(
                vec![Integer::new(0)],
                Transaction::new(
                    Address::zero(),
                    Range::new(0, 5),
                    Bytes::default(),
                    Bytes::default(),
                    Metadata::default(),
                ),
            )],
        );

        let _ = block_db.save_block(&plasma_block);
        let result = block_db.get_block(Integer::new(1));

        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.get_transactions().len(), 1);
        assert_eq!(block.get_state_updates().len(), 1);
    }

    #[test]
    fn test_abi_plasma_block() {
        let plasma_block = PlasmaBlock::new(
            1,
            vec![StateUpdate::new(
                Integer::new(7),
                Address::zero(),
                Range::new(12, 13),
                Property::new(Address::zero(), vec![]),
            )],
            vec![NewTransactionEvent::new(
                vec![Integer::new(8)],
                Transaction::new(
                    Address::zero(),
                    Range::new(14, 15),
                    Bytes::default(),
                    Bytes::default(),
                    Metadata::default(),
                ),
            )],
        );
        let decoded = PlasmaBlock::from_abi(&plasma_block.to_abi()).unwrap();

        assert_eq!(plasma_block.get_block_number(), decoded.get_block_number());
        assert_eq!(
            plasma_block.get_state_updates().len(),
            decoded.get_state_updates().len()
        );
        assert_eq!(
            plasma_block.get_transactions().len(),
            decoded.get_transactions().len()
        );
    }
}
