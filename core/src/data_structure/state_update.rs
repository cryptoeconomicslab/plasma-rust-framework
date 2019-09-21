extern crate ethabi;

use super::{Range, StateObject};
use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::Integer;
use ethabi::{ParamType, Token};
use ethereum_types::Address;

#[derive(Clone, Debug, PartialEq, Eq, AbiDecodable, AbiEncodable)]
pub struct StateUpdate {
    state_object: StateObject,
    range: Range,
    block_number: Integer,
    plasma_contract: Address,
}

impl StateUpdate {
    pub fn new(
        state_object: StateObject,
        range: Range,
        block_number: Integer,
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
    pub fn get_block_number(&self) -> Integer {
        self.block_number
    }
    pub fn get_state_object(&self) -> &StateObject {
        &self.state_object
    }
}

#[cfg(test)]
mod tests {
    use super::{Range, StateObject, StateUpdate};
    use abi_utils::{Decodable, Encodable, Integer};
    use bytes::Bytes;
    use ethereum_types::Address;

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Bytes::from(&b"parameters"[..]);
        let state_object = StateObject::new(Address::zero(), parameters_bytes);

        let state_update = StateUpdate::new(
            state_object,
            Range::new(0, 100),
            Integer(1),
            Address::zero(),
        );
        let encoded = state_update.to_abi();
        let decoded: StateUpdate = StateUpdate::from_abi(&encoded).unwrap();
        assert_eq!(
            decoded.get_range().get_start(),
            state_update.get_range().get_start()
        );
    }
}
