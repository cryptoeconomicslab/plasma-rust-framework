use crate::db::message_db::Message;
use crate::types::{Integer, Property, StateUpdate};
use abi_utils::{Decodable, Encodable, Error as AbiError, ErrorKind as AbiErrorKind};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::{Address, H256};
use plasma_core::data_structure::Range;

/// PropertyInput is attribute of Property. See further discussion https://github.com/plasma-group/ovm/issues/1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyInput {
    Placeholder(Bytes),
    ConstantAddress(Address),
    ConstantBytes(Bytes),
    ConstantH256(H256),
    ConstantInteger(Integer),
    ConstantRange(Range),
    ConstantProperty(Property),
    ConstantStateUpdate(StateUpdate),
    ConstantMessage(Message),
}

impl PropertyInput {
    pub fn placeholder(placeholder: &str) -> Self {
        PropertyInput::Placeholder(Bytes::from(placeholder))
    }
    pub fn to_address(&self) -> Address {
        if let PropertyInput::ConstantAddress(address) = self {
            *address
        } else {
            panic!("PropertyInput isn't Address!")
        }
    }
    pub fn to_state_update(&self) -> StateUpdate {
        if let PropertyInput::ConstantStateUpdate(state_update) = self {
            state_update.clone()
        } else {
            panic!("PropertyInput isn't StateUpdate!")
        }
    }
    pub fn get_type_string(&self) -> String {
        match self {
            PropertyInput::Placeholder(_) => "placeholder".to_string(),
            PropertyInput::ConstantAddress(_) => "address".to_string(),
            PropertyInput::ConstantBytes(_) => "bytes".to_string(),
            PropertyInput::ConstantH256(_) => "h256".to_string(),
            PropertyInput::ConstantInteger(_) => "integer".to_string(),
            PropertyInput::ConstantRange(_) => "range".to_string(),
            PropertyInput::ConstantProperty(property) => property.get_type_string(),
            PropertyInput::ConstantStateUpdate(_) => "state_update".to_string(),
            PropertyInput::ConstantMessage(_) => "message".to_string(),
        }
    }
}

impl Encodable for PropertyInput {
    fn to_tuple(&self) -> Vec<Token> {
        let (id, bytes) = match self {
            PropertyInput::Placeholder(placeholder) => (0, placeholder.to_vec()),
            PropertyInput::ConstantAddress(address) => (1, address.as_bytes().to_vec()),
            PropertyInput::ConstantBytes(bytes) => (2, bytes.to_vec()),
            PropertyInput::ConstantH256(h256) => (3, h256.as_bytes().to_vec()),
            PropertyInput::ConstantInteger(integer) => {
                let b: Bytes = (*integer).into();
                (4, b.to_vec())
            }
            PropertyInput::ConstantRange(range) => (5, range.to_abi()),
            PropertyInput::ConstantProperty(property) => (6, property.to_abi()),
            PropertyInput::ConstantStateUpdate(state_update) => (7, state_update.to_abi()),
            PropertyInput::ConstantMessage(message) => (8, message.to_abi()),
        };
        vec![Token::Uint(id.into()), Token::Bytes(bytes.to_vec())]
    }
}

impl Decodable for PropertyInput {
    type Ok = PropertyInput;
    fn from_tuple(tuple: &[Token]) -> Result<Self, AbiError> {
        let id = tuple[0].clone().to_uint();
        let bytes = tuple[1].clone().to_bytes();
        if let (Some(id), Some(bytes)) = (id, bytes) {
            let id_num = id.as_u64();
            if id_num == 0 {
                Ok(PropertyInput::Placeholder(Bytes::from(bytes)))
            } else if id_num == 1 {
                Ok(PropertyInput::ConstantAddress(Address::from_slice(&bytes)))
            } else if id_num == 2 {
                Ok(PropertyInput::ConstantBytes(Bytes::from(bytes)))
            } else if id_num == 3 {
                Ok(PropertyInput::ConstantH256(H256::from_slice(&bytes)))
            } else if id_num == 4 {
                Ok(PropertyInput::ConstantInteger(Bytes::from(bytes).into()))
            } else if id_num == 5 {
                Range::from_abi(&bytes).map(PropertyInput::ConstantRange)
            } else if id_num == 6 {
                Property::from_abi(&bytes).map(PropertyInput::ConstantProperty)
            } else if id_num == 7 {
                StateUpdate::from_abi(&bytes).map(PropertyInput::ConstantStateUpdate)
            } else if id_num == 8 {
                Message::from_abi(&bytes).map(PropertyInput::ConstantMessage)
            } else {
                panic!("")
            }
        } else {
            Err(AbiError::from(AbiErrorKind::AbiDecode))
        }
    }
    fn get_param_types() -> Vec<ParamType> {
        vec![ParamType::Uint(256), ParamType::Bytes]
    }
}
