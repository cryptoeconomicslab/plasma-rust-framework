use bytes::Bytes;
use plasma_core::data_structure::abi::Decodable;
use plasma_core::data_structure::error::Error as PlasmaCoreError;
use plasma_core::data_structure::{Range, Transaction};
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::traits::rangestore::RangeStore;
use plasma_db::RangeDbImpl;

pub struct TransactionDb<'a, KVS> {
    db: &'a RangeDbImpl<KVS>,
}

impl<'a, KVS> TransactionDb<'a, KVS>
where
    KVS: KeyValueStore,
{
    pub fn new(db: &'a RangeDbImpl<KVS>) -> Self {
        Self { db }
    }

    pub fn get_transactions(
        &self,
        block_number: u64,
        range: Range,
    ) -> Result<Vec<Transaction>, PlasmaCoreError> {
        let result: Result<Vec<_>, _> = self
            .db
            .bucket(&Bytes::from(&b"transaction_db"[..]))
            .bucket(&Bytes::from(format!("block_{}", block_number).as_bytes()))
            .get(range.get_start(), range.get_end())
            .ok()
            .unwrap()
            .iter()
            .map(|b| Transaction::from_abi(&b.get_value()))
            .collect();
        result
    }
}
