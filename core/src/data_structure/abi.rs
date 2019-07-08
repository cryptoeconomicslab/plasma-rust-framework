extern crate ethabi;

use super::error::Error;
use ethabi::Token;

pub trait Encodable {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }

    fn to_tuple(&self) -> Vec<Token>;
}

pub trait Decodable {
    type Ok;
    fn from_tuple(tuple: &[Token]) -> Result<Self::Ok, Error>;

    fn from_abi(data: &[u8]) -> Result<Self::Ok, Error>;
}
