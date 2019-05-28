extern crate ethabi;
extern crate rlp;

use bytes::{BufMut, BytesMut};
use ethabi::Token;
use ethereum_types::Address;

fn create_object_id(start: u64, end: u64) -> Vec<u8> {
    let mut object_id_buf = BytesMut::with_capacity(64);
    object_id_buf.put_u64_le(start);
    object_id_buf.put_u64_le(end);
    object_id_buf.to_vec()
}

#[derive(Clone, Debug, PartialEq)]
pub struct StateObject {
    id: Vec<u8>,
    predicate: Address,
    data: Vec<u8>,
}

impl StateObject {
    pub fn from_range(start: u64, end: u64, predicate: Address, data: &[u8]) -> StateObject {
        StateObject::new(&create_object_id(start, end), predicate, data)
    }
    pub fn new(object_id: &[u8], predicate: Address, data: &[u8]) -> StateObject {
        StateObject {
            id: object_id.to_vec(),
            predicate,
            data: data.to_vec(),
        }
    }
    pub fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Bytes(self.id.clone()),
            Token::Address(self.predicate),
            Token::Bytes(self.data.clone()),
        ])
    }
    pub fn from_abi(data: &[u8]) -> Result<Self, ethabi::Error> {
        let decoded: Vec<Token> = ethabi::decode(
            &[
                ethabi::ParamType::Bytes,
                ethabi::ParamType::Address,
                ethabi::ParamType::Bytes,
            ],
            data,
        )?;
        let id = decoded[0].clone().to_bytes().unwrap();
        let predicate = decoded[1].clone().to_address().unwrap();
        let data = decoded[2].clone().to_bytes().unwrap();
        Ok(StateObject::new(&id, predicate, &data))
    }
}

#[cfg(test)]
mod tests {
    use super::StateObject;
    use ethereum_types::Address;

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Vec::from(&b"parameters"[..]);
        let state_object = StateObject::from_range(0, 100, Address::zero(), &parameters_bytes);
        let encoded = state_object.to_abi();
        let decoded: StateObject = StateObject::from_abi(&encoded).unwrap();
        assert_eq!(decoded.predicate, state_object.predicate);
    }

}
