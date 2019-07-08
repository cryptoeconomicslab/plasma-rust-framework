extern crate ethabi;

use super::abi::{Decodable, Encodable};
use super::error::{Error, ErrorKind};
use super::{Range, StateObject};
use ethabi::{ParamType, Token};
use ethereum_types::Address;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateUpdate {
    state_object: StateObject,
    range: Range,
    block_number: u64,
    plasma_contract: Address,
}

impl StateUpdate {
    pub fn new(
        state_object: StateObject,
        range: Range,
        block_number: u64,
        plasma_contract: Address,
    ) -> Self {
        StateUpdate {
            state_object,
            range,
            block_number,
            plasma_contract,
        }
    }
    pub fn get_range(&self) -> &Range {
        &self.range
    }
    pub fn get_block_number(&self) -> u64 {
        self.block_number
    }
    pub fn get_state_object(&self) -> &StateObject {
        &self.state_object
    }
}

impl Encodable for StateUpdate {
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Bytes(self.state_object.to_abi()),
            Token::Tuple(self.range.to_tuple()),
            Token::Uint(self.block_number.into()),
            Token::Address(self.plasma_contract),
        ]
    }
}

impl Decodable for StateUpdate {
    type Ok = Self;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let state_object = tuple[0].clone().to_bytes();
        let range = tuple[1].clone().to_tuple();
        let block_number = tuple[2].clone().to_uint();
        let plasma_contract = tuple[3].clone().to_address();

        if let (Some(state_object), Some(range), Some(block_number), Some(plasma_contract)) =
            (state_object, range, block_number, plasma_contract)
        {
            Ok(StateUpdate::new(
                StateObject::from_abi(&state_object).unwrap(),
                Range::from_tuple(&range).unwrap(),
                block_number.as_u64(),
                plasma_contract,
            ))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded: Vec<Token> = ethabi::decode(
            &[
                // ParamType::Tuple(vec![ParamType::Address, ParamType::Bytes]),
                ParamType::Bytes,
                ParamType::Tuple(vec![ParamType::Uint(8), ParamType::Uint(8)]),
                ParamType::Uint(8),
                ParamType::Address,
            ],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

#[cfg(test)]
mod tests {
    use super::{Range, StateObject, StateUpdate};
    use crate::data_structure::abi::{Decodable, Encodable};
    use bytes::Bytes;
    use ethereum_types::Address;

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Bytes::from(&b"parameters"[..]);
        let state_object = StateObject::new(Address::zero(), parameters_bytes);

        let state_update = StateUpdate::new(state_object, Range::new(0, 100), 1, Address::zero());
        let encoded = state_update.to_abi();
        let decoded: StateUpdate = StateUpdate::from_abi(&encoded).unwrap();
        assert_eq!(
            decoded.get_range().get_start(),
            state_update.get_range().get_start()
        );
    }

}
