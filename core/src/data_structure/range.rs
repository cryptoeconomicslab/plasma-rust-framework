use super::abi::{Decodable, Encodable};
use super::error::{Error, ErrorKind};
use ethabi::Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub fn get_overlapping_range(&self, b: &Range) -> Range {
        if self.start < b.start && b.start <= self.end {
            Range::new(b.start, self.end)
        } else if b.start < self.start && self.start <= b.end {
            Range::new(self.start, b.end)
        } else {
            Range::new(0, 0)
        }
    }
}

impl Encodable for Range {
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
    fn get_param_types() -> Vec<ethabi::ParamType> {
        vec![ethabi::ParamType::Uint(64), ethabi::ParamType::Uint(64)]
    }
}
