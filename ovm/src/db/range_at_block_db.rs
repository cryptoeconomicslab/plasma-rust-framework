use crate::error::Error;
use crate::types::{Integer, PlasmaDataBlock};
use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::{Decodable, Encodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use plasma_core::data_structure::Range;
use plasma_db::prelude::*;

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct RangeAtBlockRecord {
    pub root: Bytes,
    pub inclusion_proof: Bytes,
    pub plasma_data_block: PlasmaDataBlock,
}

impl RangeAtBlockRecord {
    pub fn new(root: Bytes, inclusion_proof: Bytes, plasma_data_block: PlasmaDataBlock) -> Self {
        Self {
            root,
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
        root: Bytes,
        inclusion_proof: Bytes,
        plasma_data_block: PlasmaDataBlock,
    ) -> Result<(), Error> {
        let record = RangeAtBlockRecord::new(root, inclusion_proof, plasma_data_block.clone());
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
    pub fn get_witness(
        &self,
        block_number: Integer,
        coin_range: Range,
    ) -> Result<RangeAtBlockRecord, Error> {
        let result = self
            .db
            .bucket(&Bytes::from(&b"range_at_block"[..]))
            .bucket(&block_number.into())
            .get(coin_range.get_start(), coin_range.get_end())
            .map_err::<Error, _>(Into::into)?;
        if result.len() == 0 {
            panic!("inclusion proof not found");
        }
        RangeAtBlockRecord::from_abi(&result[0].get_value()).map_err::<Error, _>(Into::into)
    }
}
