use crate::types::Integer;
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::{Address, H256};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};
use plasma_core::data_structure::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputType {
    Placeholder(Bytes),
    ConstantAddress(Address),
    ConstantBytes(Bytes),
    ConstantH256(H256),
    ConstantInteger(Integer),
    ConstantRange(Range),
}

impl InputType {
    pub fn placeholder(placeholder: &str) -> Self {
        InputType::Placeholder(Bytes::from(placeholder))
    }
}

impl Encodable for InputType {
    fn to_tuple(&self) -> Vec<Token> {
        match self {
            InputType::Placeholder(placeholder) => {
                vec![Token::Uint(0.into()), Token::Bytes(placeholder.to_vec())]
            }
            InputType::ConstantAddress(address) => vec![
                Token::Uint(1.into()),
                Token::Bytes(address.as_bytes().to_vec()),
            ],
            InputType::ConstantBytes(bytes) => {
                vec![Token::Uint(2.into()), Token::Bytes(bytes.to_vec())]
            }
            InputType::ConstantH256(h256) => vec![
                Token::Uint(3.into()),
                Token::Bytes(h256.as_bytes().to_vec()),
            ],
            InputType::ConstantInteger(integer) => {
                let b: Bytes = (*integer).into();
                vec![Token::Uint(4.into()), Token::Bytes(b.to_vec())]
            }
            InputType::ConstantRange(range) => {
                vec![Token::Uint(5.into()), Token::Bytes(range.to_abi())]
            }
        }
    }
}

impl Decodable for InputType {
    type Ok = InputType;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let id = tuple[0].clone().to_uint();
        let bytes = tuple[1].clone().to_bytes();
        if let (Some(id), Some(bytes)) = (id, bytes) {
            let id_num = id.as_u64();
            if id_num == 0 {
                Ok(InputType::Placeholder(Bytes::from(bytes)))
            } else if id_num == 1 {
                Ok(InputType::ConstantAddress(Address::from_slice(&bytes)))
            } else if id_num == 2 {
                Ok(InputType::ConstantBytes(Bytes::from(bytes)))
            } else if id_num == 3 {
                Ok(InputType::ConstantH256(H256::from_slice(&bytes)))
            } else if id_num == 4 {
                Ok(InputType::ConstantInteger(Bytes::from(bytes).into()))
            } else {
                Range::from_abi(&bytes).map(InputType::ConstantRange)
            }
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn get_param_types() -> Vec<ParamType> {
        vec![ParamType::Uint(256), ParamType::Bytes]
    }
}
