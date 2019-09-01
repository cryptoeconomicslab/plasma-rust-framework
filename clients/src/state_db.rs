use bytes::Bytes;
use ovm::types::PlasmaDataBlock;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_db::error::Error;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::traits::rangestore::RangeStore;
use plasma_db::RangeDbImpl;

pub struct StateDb<'a, KVS: KeyValueStore> {
    db: &'a RangeDbImpl<KVS>,
}

impl<'a, KVS: KeyValueStore> StateDb<'a, KVS> {
    pub fn new(db: &'a RangeDbImpl<KVS>) -> Self {
        StateDb { db }
    }

    pub fn get_verified_plasma_data_blocks(
        &self,
        start: u64,
        end: u64,
    ) -> Result<Vec<PlasmaDataBlock>, Error> {
        let res = self
            .db
            .bucket(&Bytes::from(&b"verified_plasma_data_blocks"[..]))
            .get(start, end)?
            .iter()
            .map(|range| PlasmaDataBlock::from_abi(range.get_value()).unwrap())
            .collect();
        println!("{:?}", res);

        Ok(res)
    }

    pub fn put_verified_plasma_data_block(
        &mut self,
        plasma_data_block: PlasmaDataBlock,
    ) -> Result<(), Error> {
        let start = plasma_data_block.get_updated_range().get_start();
        let end = plasma_data_block.get_updated_range().get_end();

        let result = self
            .db
            .bucket(&Bytes::from(&b"verified_plasma_data_blocks"[..]))
            .get(start, end)?;
        println!("{:?}", result);
        if result.len() == 0 {
            // TODO: handle error
            let _ = self.db
                .bucket(&Bytes::from(&b"verified_plasma_data_blocks"[..]))
                .put(start, end, &plasma_data_block.to_abi());
            Ok(())
        } else {
            // TODO: update, split, override
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StateDb;
    use bytes::Bytes;
    use ethereum_types::Address;
    use ovm::types::{Integer, PlasmaDataBlock, Property, SignedByInput};
    use plasma_core::data_structure::Range;
    use plasma_db::impls::kvs::memory::CoreDbMemoryImpl;
    use plasma_db::traits::db::DatabaseTrait;
    use plasma_db::RangeDbImpl;

    #[test]
    fn test_state_db() {
        let base_db = CoreDbMemoryImpl::open("test");
        let db = RangeDbImpl::from(base_db);
        let mut state_db = StateDb::new(&db);
        let address: Address = Address::zero();

        let plasma_data_block = PlasmaDataBlock::new(
            Integer::new(1),
            Range::new(0, 100),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
            Bytes::from(&b"root"[..])
        );

        let _ = state_db.put_verified_plasma_data_block(plasma_data_block);
        let result = state_db
            .get_verified_plasma_data_blocks(0, 100)
            .unwrap();
        assert_eq!(result.len(), 1);
    }
}
