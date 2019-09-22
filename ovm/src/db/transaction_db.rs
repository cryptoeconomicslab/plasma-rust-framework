use super::TransactionFilter;
use abi_utils::{Decodable, Encodable, Error as AbiError};
use bytes::Bytes;
use plasma_core::data_structure::{Range, Transaction};
use plasma_db::{
    traits::{kvs::KeyValueStore, rangestore::RangeStore},
    RangeDbImpl,
};

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
    ) -> Result<Vec<Transaction>, AbiError> {
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

    pub fn put_transaction(&self, block_number: u64, transaction: Transaction) {
        let range = transaction.get_range();
        let _ = self
            .db
            .bucket(&Bytes::from(&b"transaction_db"[..]))
            .bucket(&Bytes::from(format!("block_{}", block_number).as_bytes()))
            .put(range.get_start(), range.get_end(), &transaction.to_abi());
    }

    pub fn query_transaction(
        &self,
        transaction_filter: TransactionFilter,
    ) -> Result<Vec<Transaction>, AbiError> {
        let from_block = transaction_filter.get_from_block();
        let to_block = transaction_filter.get_to_block();
        let range = transaction_filter.get_range();
        let mut result = vec![];
        for i in from_block..=to_block {
            let txs = self.get_transactions(i, range)?;
            println!("{:?}", txs);
            result.append(&mut transaction_filter.query(txs));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::super::TransactionFilterBuilder;
    use super::*;
    use bytes::Bytes;
    use ethereum_types::Address;
    use plasma_core::data_structure::{Metadata, Range};
    use plasma_db::{impls::kvs::CoreDbMemoryImpl, traits::DatabaseTrait, RangeDbImpl};

    #[test]
    fn test_query_transaction() {
        let db = CoreDbMemoryImpl::open("test");
        let range_db = RangeDbImpl::from(db);

        let tx_db = TransactionDb::new(&range_db);
        let address =
            Address::from_slice(&hex::decode("2932b7a2355d6fecc4b5c0b6bd44cc31df247a2e").unwrap());

        for i in 0..5 {
            tx_db.put_transaction(
                i,
                Transaction::new(
                    Address::zero(),
                    Range::new(i % 3, i),
                    Bytes::default(),
                    Bytes::default(),
                    Metadata::new(
                        if i % 2 == 0 { Address::zero() } else { address },
                        Address::zero(),
                    ),
                ),
            );
        }

        let filter = TransactionFilterBuilder::new()
            .range(Range::new(2, 5))
            .block_from(1)
            .block_to(4)
            .address_from(address)
            .build();

        let result = tx_db.query_transaction(filter);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}
