use super::abi::{Decodable, Encodable};
use super::error::{Error, ErrorKind};
use ethabi::Token;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    start: u64,
    end: u64,
}

impl Range {
    pub fn new(start: u64, end: u64) -> Self {
        Range { start, end }
    }
    pub fn get_start(&self) -> u64 {
        self.start
    }
    pub fn get_end(&self) -> u64 {
        self.end
    }
}

impl Encodable for Range {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![Token::Uint(self.start.into()), Token::Uint(self.end.into())]
    }
}

impl Decodable for Range {
    type Ok = Self;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let start = tuple[0].clone().to_uint();
        let end = tuple[1].clone().to_uint();
        if let (Some(start), Some(end)) = (start, end) {
            Ok(Range::new(start.as_u64(), end.as_u64()))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded: Vec<Token> = ethabi::decode(
            &[ethabi::ParamType::Uint(8), ethabi::ParamType::Uint(8)],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}
