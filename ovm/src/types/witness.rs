use super::core::Property;
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::U256;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};
use plasma_core::data_structure::Range;

#[derive(Clone, Debug)]
pub struct PlasmaDataBlock {
    updated_range: Range,
    property: Property,
}

impl PlasmaDataBlock {
    pub fn get_updated_range(&self) -> Range {
        self.updated_range.clone()
    }
    pub fn get_property(&self) -> &Property {
        &self.property
    }
}

impl Encodable for PlasmaDataBlock {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Tuple(self.updated_range.to_tuple()),
            Token::Tuple(self.property.to_tuple()),
        ]
    }
}

impl Decodable for PlasmaDataBlock {
    type Ok = PlasmaDataBlock;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let updated_range = tuple[0].clone().to_tuple();
        let property = tuple[1].clone().to_bytes();
        if let (Some(updated_range), Some(property)) = (updated_range, property) {
            Ok(PlasmaDataBlock {
                updated_range: Range::from_tuple(&updated_range).unwrap(),
                property: Property::from_abi(&property).unwrap(),
            })
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, PlasmaCoreError> {
        let decoded = ethabi::decode(
            &[
                ParamType::Tuple(vec![ParamType::Uint(64), ParamType::Uint(64)]),
                ParamType::Bytes,
            ],
            data,
        )
        .map_err(|_e| PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

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
                Token::Bytes(plasma_data_block.to_abi()),
            ],
        }
    }
    fn from_tuple_part(id: u64, tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        if id == 0 {
            let bytes = tuple[0].clone().to_bytes();
            if let Some(bytes) = bytes {
                Ok(Witness::Bytes(Bytes::from(bytes)))
            } else {
                Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
            }
        } else {
            let inclusion_proof = tuple[0].clone().to_bytes();
            let plasma_data_block = tuple[1].clone().to_bytes();
            if let (Some(inclusion_proof), Some(plasma_data_block)) =
                (inclusion_proof, plasma_data_block)
            {
                Ok(Witness::IncludedInIntervalTreeAtBlock(
                    Bytes::from(inclusion_proof),
                    PlasmaDataBlock::from_abi(&plasma_data_block).unwrap(),
                ))
            } else {
                Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
            }
        }
    }
    fn from_abi_part(id: U256, data: &[u8]) -> Result<Self, PlasmaCoreError> {
        let decoded = ethabi::decode(&Self::get_param_types(id.as_u64()), data)
            .map_err(|_e| PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))?;
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
            vec![ParamType::Bytes, ParamType::Bytes]
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
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, PlasmaCoreError> {
        let decoded = ethabi::decode(&[ParamType::Uint(256), ParamType::Bytes], data)
            .map_err(|_e| PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}
