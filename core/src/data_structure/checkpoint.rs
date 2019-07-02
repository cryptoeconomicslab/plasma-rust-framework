extern crate ethabi;
extern crate rlp;

use super::error::{Error, ErrorKind};
use super::state_object::StateObject;
use ethabi::Token;
use ethereum_types::Address;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    state_update: StateUpdate,
    start: u64,
    end: u64,
}

impl Checkpoint {
    pub fn new(
        state_update: StateUpdate,
        start: u64,
        end: u64,
    ) -> Self {
        Self {
            state_update,
            start,
            end,
        }
    }
    pub fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let state_update_tuple = tuple[0].clone().to_tuple();
        let range_tuple = tuple[1].clone().to_tuple();
        
        if let (
            Some(state_update_tuple),
            Some(range_tuple),
        ) = (state_update_tuple, range_tuple)
        {
            Ok(Checkpoint::new(
                StateUpdate::from_tuple(state_update_tuple),
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
    pub fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded: Vec<Token> = ethabi::decode(
            &[ParamType::Tuple(vec![
                ParamType::Tuple(vec![
                    ParamType::Tuple(vec![ParamType::Address, ParamType::Bytes]),
                    ParamType::Tuple(vec![ParamType::Uint(32), ParamType::Uint(32)]),
                    ParamType::Uint(32),
                    ParamType::Address,
                ]),
                ParamType::Tuple(vec![ParamType::Uint(32), ParamType::Uint(32)]),
            ])],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
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
    use bytes::Bytes;
    use ethereum_types::Address;

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Bytes::from(&b"parameters"[..]);
        let state_object = StateObject::new(Address::zero(), parameters_bytes);

        let state_update = StateUpdate::new(state_object, 0, 100, 1, Address::zero());
        let encoded = state_update.to_abi();
        let decoded: StateUpdate = StateUpdate::from_abi(&encoded).unwrap();
        assert_eq!(decoded.start, state_update.start);
    }

}
