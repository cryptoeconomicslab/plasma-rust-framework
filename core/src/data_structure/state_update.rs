extern crate ethabi;
extern crate rlp;

use super::error::{Error, ErrorKind};
use super::state_object::StateObject;
use ethabi::Token;
use ethereum_types::Address;

#[derive(Clone, Debug, PartialEq)]
pub struct StateUpdate {
    state_object: StateObject,
    start: u64,
    end: u64,
    block_number: u64,
    plasma_contract: Address,
}

impl StateUpdate {
    pub fn new(
        state_object: StateObject,
        start: u64,
        end: u64,
        block_number: u64,
        plasma_contract: Address,
    ) -> Self {
        StateUpdate {
            state_object,
            start,
            end,
            block_number,
            plasma_contract,
        }
    }
    pub fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Bytes(self.state_object.to_abi()),
            Token::Uint(self.start.into()),
            Token::Uint(self.end.into()),
            Token::Uint(self.block_number.into()),
            Token::Address(self.plasma_contract),
        ])
    }
    pub fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded: Vec<Token> = ethabi::decode(
            &[
                ethabi::ParamType::Bytes,
                ethabi::ParamType::Uint(8),
                ethabi::ParamType::Uint(8),
                ethabi::ParamType::Uint(8),
                ethabi::ParamType::Address,
            ],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        let state_object = decoded[0].clone().to_bytes();
        let start = decoded[1].clone().to_uint();
        let end = decoded[2].clone().to_uint();
        let block_number = decoded[3].clone().to_uint();
        let plasma_contract = decoded[4].clone().to_address();

        if let (
            Some(state_object),
            Some(start),
            Some(end),
            Some(block_number),
            Some(plasma_contract),
        ) = (state_object, start, end, block_number, plasma_contract)
        {
            Ok(StateUpdate::new(
                StateObject::from_abi(&state_object).unwrap(),
                start.as_u64(),
                end.as_u64(),
                block_number.as_u64(),
                plasma_contract,
            ))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    pub fn get_start(&self) -> u64 {
        self.start
    }
    pub fn get_end(&self) -> u64 {
        self.end
    }
    pub fn get_block_number(&self) -> u64 {
        self.block_number
    }
    pub fn get_state_object(&self) -> &StateObject {
        &self.state_object
    }
}

#[cfg(test)]
mod tests {
    use super::StateObject;
    use super::StateUpdate;
    use ethereum_types::Address;

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Vec::from(&b"parameters"[..]);
        let state_object = StateObject::new(Address::zero(), &parameters_bytes);

        let state_update = StateUpdate::new(state_object, 0, 100, 1, Address::zero());
        let encoded = state_update.to_abi();
        let decoded: StateUpdate = StateUpdate::from_abi(&encoded).unwrap();
        assert_eq!(decoded.start, state_update.start);
    }

}
