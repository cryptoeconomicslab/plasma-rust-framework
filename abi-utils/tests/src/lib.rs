use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::Integer;
use bytes::Bytes;
use ethabi::{ParamType, Token};

#[derive(Clone, Debug, AbiEncodable, AbiDecodable)]
pub struct TestStruct {
    pub integer: Integer,
    bytes: Bytes,
    pub array: Vec<Integer>,
}

impl TestStruct {
    pub fn new(integer: Integer, bytes: Bytes, array: Vec<Integer>) -> Self {
        Self {
            integer,
            bytes,
            array,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi_utils::*;

    #[test]
    fn test_abi_encode() {
        let test_struct = TestStruct::new(Integer(10), Bytes::from("aaaaa"), vec![Integer(5)]);
        let encoded = test_struct.to_abi();
        let decoded: TestStruct = TestStruct::from_abi(&encoded).unwrap();
        assert_eq!(decoded.integer, test_struct.integer);
    }
}
