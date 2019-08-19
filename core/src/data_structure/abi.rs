extern crate ethabi;

use super::error::{Error, ErrorKind};
use ethabi::{ParamType, Token};

pub trait Encodable {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }

    fn to_tuple(&self) -> Vec<Token>;
}

pub trait Decodable {
    type Ok;
    fn from_tuple(tuple: &[Token]) -> Result<Self::Ok, Error>;

    fn from_abi(data: &[u8]) -> Result<Self::Ok, Error> {
        let decoded = ethabi::decode(&Self::get_param_types(), data)
            .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
    fn get_param_types() -> Vec<ParamType> {
        vec![]
    }
}
