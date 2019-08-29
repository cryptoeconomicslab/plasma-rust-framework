use crate::error::Error;
use crate::types::{IncludedAtBlockInput, Witness};
use bytes::Bytes;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::traits::rangestore::RangeStore;
use plasma_db::RangeDbImpl;

pub struct RangeAtBlockDb<'a, KVS: KeyValueStore> {
    db: &'a RangeDbImpl<KVS>,
}

impl<'a, KVS: KeyValueStore> RangeAtBlockDb<'a, KVS> {
    pub fn new(db: &'a RangeDbImpl<KVS>) -> Self {
        Self { db }
    }
    pub fn store_witness(
        &self,
        input: &IncludedAtBlockInput,
        witness: &Witness,
    ) -> Result<(), Error> {
        self.db
            .bucket(&Bytes::from(&b"range_at_block"[..]))
            .bucket(&input.get_block_number().into())
            .put(
                input
                    .get_plasma_data_block()
                    .get_updated_range()
                    .get_start(),
                input.get_plasma_data_block().get_updated_range().get_end(),
                &witness.to_abi(),
            )
            .map_err::<Error, _>(Into::into)
    }
    pub fn get_witness(&self, input: &IncludedAtBlockInput) -> Result<Witness, Error> {
        let result = self
            .db
            .bucket(&Bytes::from(&b"range_at_block"[..]))
            .bucket(&input.get_block_number().into())
            .get(
                input
                    .get_plasma_data_block()
                    .get_updated_range()
                    .get_start(),
                input.get_plasma_data_block().get_updated_range().get_end(),
            )
            .map_err::<Error, _>(Into::into)?;
        if result.len() == 0 {
            panic!("inclusion proof not found");
        }
        Witness::from_abi(&result[0].get_value()).map_err::<Error, _>(Into::into)
    }
}
