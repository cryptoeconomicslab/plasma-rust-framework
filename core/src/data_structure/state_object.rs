extern crate ethabi;
extern crate rlp;

use super::error::{Error, ErrorKind};
use ethabi::Token;
use ethereum_types::Address;

/*
fn create_object_id(start: u64, end: u64) -> Vec<u8> {
    let mut object_id_buf = BytesMut::with_capacity(64);
    object_id_buf.put_u64_le(start);
    object_id_buf.put_u64_le(end);
    object_id_buf.to_vec()
}
*/

#[derive(Clone, Debug, PartialEq)]
pub struct StateObject {
    predicate: Address,
    data: Vec<u8>,
}

impl StateObject {
    pub fn new(predicate: Address, data: &[u8]) -> StateObject {
        StateObject {
            predicate,
            data: data.to_vec(),
        }
    }
    pub fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Address(self.predicate),
            Token::Bytes(self.data.clone()),
        ])
    }
    pub fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let predicate = tuple[0].clone().to_address();
        let data = tuple[1].clone().to_bytes();
        if let (Some(predicate), Some(data)) = (predicate, data) {
            Ok(StateObject::new(predicate, &data))
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
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::StateObject;
    use ethereum_types::Address;

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Vec::from(&b"parameters"[..]);
        let state_object = StateObject::new(Address::zero(), &parameters_bytes);
        let encoded = state_object.to_abi();
        let decoded: StateObject = StateObject::from_abi(&encoded).unwrap();
        assert_eq!(decoded.predicate, state_object.predicate);
    }

}
