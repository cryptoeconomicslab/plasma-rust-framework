use super::core::Integer;
// TODO: use general verifier.
// use super::super::deciders::SignVerifier;
use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use plasma_core::data_structure::Range;

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct PlasmaDataBlock {
    deposit_contract_address: Address,
    updated_range: Range,
    root: Bytes,
    is_included: bool,
    block_number: Integer,
    data: Bytes,
}

impl PlasmaDataBlock {
    pub fn new(
        deposit_contract_address: Address,
        updated_range: Range,
        root: Bytes,
        is_included: bool,
        block_number: Integer,
        data: Bytes,
    ) -> Self {
        Self {
            deposit_contract_address,
            updated_range,
            root,
            is_included,
            block_number,
            data,
        }
    }
    pub fn get_deposit_contract_address(&self) -> Address {
        self.deposit_contract_address
    }
    pub fn get_updated_range(&self) -> Range {
        self.updated_range
    }

    pub fn set_updated_range(&mut self, range: Range) {
        self.updated_range = range;
    }

    pub fn get_is_included(&self) -> bool {
        self.is_included
    }
    pub fn get_root(&self) -> &Bytes {
        &self.root
    }
    pub fn get_block_number(&self) -> Integer {
        self.block_number
    }
    pub fn get_data(&self) -> &Bytes {
        &self.data
    }
}
