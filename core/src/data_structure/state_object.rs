extern crate ethabi;

use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;

#[derive(Clone, Debug, PartialEq, Eq, AbiDecodable, AbiEncodable)]
/// StateObject represents state of assets
/// See http://spec.plasma.group/en/latest/src/01-core/state-system.html#state-objects
pub struct StateObject {
    predicate: Address,
    data: Bytes,
}

impl StateObject {
    pub fn new(predicate: Address, data: Bytes) -> StateObject {
        StateObject { predicate, data }
    }
    pub fn get_predicate(&self) -> Address {
        self.predicate
    }
    pub fn get_data(&self) -> &Bytes {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::StateObject;
    use abi_utils::{Decodable, Encodable};
    use bytes::Bytes;
    use ethereum_types::Address;

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Bytes::from(&b"parameters"[..]);
        let state_object = StateObject::new(Address::zero(), parameters_bytes);
        let encoded = state_object.to_abi();
        let decoded: StateObject = StateObject::from_abi(&encoded).unwrap();
        assert_eq!(decoded.predicate, state_object.predicate);
    }
}
