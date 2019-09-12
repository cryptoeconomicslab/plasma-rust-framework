use crate::error::Error;
use crate::types::{IncludedAtBlockInput, PlasmaDataBlock};
use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::traits::rangestore::RangeStore;
use plasma_db::RangeDbImpl;

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct RangeAtBlockRecord {
    pub inclusion_proof: Bytes,
    pub plasma_data_block: PlasmaDataBlock,
}

impl RangeAtBlockRecord {
    pub fn new(inclusion_proof: Bytes, plasma_data_block: PlasmaDataBlock) -> Self {
        Self {
            inclusion_proof,
            plasma_data_block,
        }
    }
}

pub struct RangeAtBlockDb<'a, KVS: KeyValueStore> {
    db: &'a RangeDbImpl<KVS>,
}

impl<'a, KVS: KeyValueStore> RangeAtBlockDb<'a, KVS> {
    pub fn new(db: &'a RangeDbImpl<KVS>) -> Self {
        Self { db }
    }
    pub fn store_witness(
        &self,
        inclusion_proof: Bytes,
        plasma_data_block: PlasmaDataBlock,
    ) -> Result<(), Error> {
        let record = RangeAtBlockRecord::new(inclusion_proof, plasma_data_block.clone());
        let block_number = plasma_data_block.get_block_number();
        self.db
            .bucket(&Bytes::from(&b"range_at_block"[..]))
            .bucket(&block_number.into())
            .put(
                plasma_data_block.get_updated_range().get_start(),
                plasma_data_block.get_updated_range().get_end(),
                &record.to_abi(),
            )
            .map_err::<Error, _>(Into::into)
    }
    pub fn get_witness(&self, input: &IncludedAtBlockInput) -> Result<RangeAtBlockRecord, Error> {
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
        RangeAtBlockRecord::from_abi(&result[0].get_value()).map_err::<Error, _>(Into::into)
    }
}
