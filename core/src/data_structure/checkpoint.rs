extern crate ethabi;

use super::abi::{Decodable, Encodable};
use super::error::{Error, ErrorKind};
use crate::data_structure::{Range, StateUpdate};
use ethabi::{ParamType, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    state_update: StateUpdate,
    range: Range,
}

impl Checkpoint {
    pub fn new(state_update: StateUpdate, range: Range) -> Self {
        Self {
            state_update,
            range,
        }
    }
    pub fn get_range(&self) -> &Range {
        &self.range
    }
    pub fn get_state_update(&self) -> &StateUpdate {
        &self.state_update
    }
}

impl Encodable for Checkpoint {
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Bytes(self.state_update.to_abi()),
            Token::Tuple(self.range.to_tuple()),
        ]
    }
}

impl Decodable for Checkpoint {
    type Ok = Self;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let state_update_tuple = tuple[0].clone().to_bytes();
        let range_tuple = tuple[1].clone().to_tuple();

        if let (Some(state_update_tuple), Some(range_tuple)) = (state_update_tuple, range_tuple) {
            Ok(Checkpoint::new(
                StateUpdate::from_abi(&state_update_tuple).unwrap(),
                Range::from_tuple(&range_tuple).unwrap(),
            ))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        println!("{:?}", data);
        let decoded: Vec<Token> = ethabi::decode(
            &[
                /*
                ParamType::Tuple(vec![
                    // ParamType::Tuple(vec![ParamType::Address, ParamType::Bytes]),
                    ParamType::Bytes,
                    ParamType::Tuple(vec![ParamType::Uint(8), ParamType::Uint(8)]),
                    ParamType::Uint(8),
                    ParamType::Address,
                ]),*/
                ParamType::Bytes,
                ParamType::Tuple(vec![ParamType::Uint(8), ParamType::Uint(8)]),
            ],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

#[cfg(test)]
mod tests {
    use crate::data_structure::{
        abi::{Decodable, Encodable},
        Checkpoint, Range, StateObject, StateUpdate,
    };
    use bytes::Bytes;
    use ethereum_types::Address;

    #[test]
    fn test_abi_encode() {
        let state_object = StateObject::new(Address::zero(), Bytes::from(&b"data"[..]));
        let state_update = StateUpdate::new(state_object, Range::new(0, 100), 1, Address::zero());
        let checkpoint = Checkpoint::new(state_update, Range::new(0, 50));
        let encoded = checkpoint.to_abi();
        let decoded = Checkpoint::from_abi(&encoded).unwrap();
        assert_eq!(
            decoded.get_range().get_start(),
            checkpoint.get_range().get_start()
        );
    }

}
