use super::core::{Integer, Property};
// TODO: use general verifier.
use super::super::deciders::SignVerifier;
use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::{Address, U256};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};
use plasma_core::data_structure::{Range, Transaction};

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
            block_number: block_number,
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

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Witness {
    Bytes(Bytes),
    // inclusion proof and plasma data block
    IncludedInIntervalTreeAtBlock(Bytes, PlasmaDataBlock),
}

impl Witness {
    pub fn to_abi_part(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple_part())
    }
    pub fn to_tuple_part(&self) -> Vec<Token> {
        match self {
            Witness::Bytes(bytes) => vec![Token::Bytes(bytes.to_vec())],
            Witness::IncludedInIntervalTreeAtBlock(inclusion_proof, plasma_data_block) => vec![
                Token::Bytes(inclusion_proof.to_vec()),
                Token::Tuple(plasma_data_block.to_tuple()),
            ],
        }
    }
    fn from_tuple_part(id: u64, tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        if id == 0 {
            let bytes = tuple[0].clone().to_bytes();
            if let Some(bytes) = bytes {
                Ok(Witness::Bytes(Bytes::from(bytes)))
            } else {
                println!("decodeing witness error");
                Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
            }
        } else {
            let inclusion_proof = tuple[0].clone().to_bytes();
            let plasma_data_block = tuple[1].clone().to_tuple();
            if let (Some(inclusion_proof), Some(plasma_data_block)) =
                (inclusion_proof, plasma_data_block)
            {
                Ok(Witness::IncludedInIntervalTreeAtBlock(
                    Bytes::from(inclusion_proof),
                    PlasmaDataBlock::from_tuple(&plasma_data_block).unwrap(),
                ))
            } else {
                println!("decodeing witness error 1");
                Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
            }
        }
    }
    fn from_abi_part(id: U256, data: &[u8]) -> Result<Self, PlasmaCoreError> {
        let decoded = ethabi::decode(&Self::get_param_types(id.as_u64()), data).map_err(|_e| {
            println!("decodeing witness error 2");
            PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode)
        })?;
        Self::from_tuple_part(id.as_u64(), &decoded)
    }
    pub fn get_number(&self) -> U256 {
        match self {
            Witness::Bytes(_) => U256::from(0),
            Witness::IncludedInIntervalTreeAtBlock(_, _) => U256::from(1),
        }
    }
    fn get_param_types(id: u64) -> Vec<ParamType> {
        if id == 0 {
            vec![ParamType::Bytes]
        } else {
            vec![
                ParamType::Bytes,
                ParamType::Tuple(PlasmaDataBlock::get_param_types()),
            ]
        }
    }
}

impl Encodable for Witness {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Uint(self.get_number()),
            Token::Bytes(self.to_abi_part()),
        ]
    }
}

impl Decodable for Witness {
    type Ok = Witness;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let witness_id = tuple[0].clone().to_uint();
        let witness_data = tuple[1].clone().to_bytes();
        if let (Some(witness_id), Some(witness_data)) = (witness_id, witness_data) {
            Ok(Witness::from_abi_part(witness_id, &witness_data).unwrap())
        } else {
            println!("decodeing witness error");
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn get_param_types() -> Vec<ParamType> {
        vec![ParamType::Uint(256), ParamType::Bytes]
    }
}
