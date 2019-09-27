use crate::error::Error;
use crate::types::{Integer, StateUpdate};
use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::{Decodable, Encodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use plasma_core::data_structure::Range;
use plasma_db::prelude::*;

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct RangeAtBlockRecord {
    pub root: Bytes,
    pub is_included: bool,
    pub inclusion_proof: Bytes,
    pub state_update: StateUpdate,
}

impl RangeAtBlockRecord {
    pub fn new(
        root: Bytes,
        is_included: bool,
        inclusion_proof: Bytes,
        state_update: StateUpdate,
    ) -> Self {
        Self {
            root,
            is_included,
            inclusion_proof,
            state_update,
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
        is_included: bool,
        inclusion_proof: Bytes,
        state_update: StateUpdate,
    ) -> Result<(), Error> {
        let record =
            RangeAtBlockRecord::new(root, is_included, inclusion_proof, state_update.clone());
        let block_number = state_update.get_block_number();
        self.db
            .bucket(&Bytes::from(&b"range_at_block"[..]))
            .bucket(&block_number.into())
            .bucket(&Bytes::from(
                state_update.get_deposit_contract_address().as_bytes(),
            ))
            .put(
                state_update.get_range().get_start(),
                state_update.get_range().get_end(),
                &record.to_abi(),
            )
            .map_err::<Error, _>(Into::into)
    }
    pub fn get_witness(
        &self,
        block_number: Integer,
        deposit_contract_address: Address,
        coin_range: Range,
    ) -> Result<RangeAtBlockRecord, Error> {
        let result = self
            .db
            .bucket(&Bytes::from(&b"range_at_block"[..]))
            .bucket(&block_number.into())
            .bucket(&Bytes::from(deposit_contract_address.as_bytes()))
            .get(coin_range.get_start(), coin_range.get_end())
            .map_err::<Error, _>(Into::into)?;
        if result.len() == 0 {
            panic!("inclusion proof not found");
        }
        RangeAtBlockRecord::from_abi(&result[0].get_value()).map_err::<Error, _>(Into::into)
    }
}
