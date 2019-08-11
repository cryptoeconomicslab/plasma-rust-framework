use super::core::Property;
use bytes::Bytes;
use ethabi::Token;
use plasma_core::data_structure::abi::Encodable;
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
}

#[derive(Clone, Debug)]
pub enum Witness {
    Bytes(Bytes),
    IncludedInIntervalTreeAtBlock(Bytes, PlasmaDataBlock),
}

impl Encodable for Witness {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        match self {
            Witness::Bytes(bytes) => vec![Token::Bytes(bytes.to_vec())],
            _ => vec![],
        }
    }
}
