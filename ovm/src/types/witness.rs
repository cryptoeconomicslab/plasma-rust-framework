use super::core::Integer;
// TODO: use general verifier.
// use super::super::deciders::SignVerifier;
use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::Range;

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct PlasmaDataBlock {
    index: Integer,
    updated_range: Range,
    root: Bytes,
    is_included: bool,
    predicate_address: Address,
    block_number: Integer,
    data: Bytes,
}

impl PlasmaDataBlock {
    pub fn new(
        index: Integer,
        updated_range: Range,
        root: Bytes,
        is_included: bool,
        predicate_address: Address,
        block_number: Integer,
        data: Bytes,
    ) -> Self {
        Self {
            index,
            updated_range,
            root,
            is_included,
            predicate_address,
            block_number,
            data,
        }
    }
    pub fn get_index(&self) -> usize {
        self.index.0 as usize
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
    pub fn get_decider_id(&self) -> Address {
        self.predicate_address
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

    //    pub fn verify_deprecation(&self, transaction: &Transaction) -> bool {
    //        if let Property::SignedByDecider(input) = &self.property {
    //            if SignVerifier::recover(
    //                transaction.get_signature(),
    //                &Bytes::from(transaction.to_body_abi()),
    //            ) == input.get_public_key()
    //            {
    //                return true;
    //            }
    //            false
    //        } else {
    //            // TODO: implement how to verify_deprecation using other.decider
    //            false
    //        }
    //    }
}

pub type Witness = Bytes;
