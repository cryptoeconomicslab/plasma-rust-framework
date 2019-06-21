extern crate ethabi;
extern crate rlp;

use super::error::{Error, ErrorKind};
use bytes::Bytes;
use ethabi::Token;
use ethereum_types::Address;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Address(self.predicate),
            Token::Bytes(self.data.to_vec()),
        ])
    }
    pub fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let predicate = tuple[0].clone().to_address();
        let data = tuple[1].clone().to_bytes();
        if let (Some(predicate), Some(data)) = (predicate, data) {
            Ok(StateObject::new(predicate, Bytes::from(data)))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    pub fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded: Vec<Token> = ethabi::decode(
            &[ethabi::ParamType::Address, ethabi::ParamType::Bytes],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
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
