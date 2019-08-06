use crate::types::{Integer, QuantifierResult};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{Error, ErrorKind};

fn get_range(start: Integer, end: Integer) -> Vec<Bytes> {
    (start.0..end.0)
        .map(|n| Bytes::from(Integer::new(n).to_abi()))
        .collect()
}

#[derive(Clone, Debug)]
pub struct IntegerRangeParameters {
    start: u64,
    end: u64,
}

impl IntegerRangeParameters {
    pub fn new(start: u64, end: u64) -> Self {
        IntegerRangeParameters { start, end }
    }
    pub fn get_start(&self) -> u64 {
        self.start
    }
    pub fn get_end(&self) -> u64 {
        self.end
    }
}

impl Encodable for IntegerRangeParameters {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![Token::Uint(self.start.into()), Token::Uint(self.end.into())]
    }
}

impl Decodable for IntegerRangeParameters {
    type Ok = IntegerRangeParameters;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let start = tuple[0].clone().to_uint();
        let end = tuple[1].clone().to_uint();
        if let (Some(start), Some(end)) = (start, end) {
            Ok(IntegerRangeParameters::new(start.as_u64(), end.as_u64()))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded = ethabi::decode(&[ParamType::Uint(256), ParamType::Uint(256)], data)
            .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

/// IntegerRangeQuantifier quantify specific range
pub struct IntegerRangeQuantifier {}

impl Default for IntegerRangeQuantifier {
    fn default() -> Self {
        IntegerRangeQuantifier {}
    }
}

impl IntegerRangeQuantifier {
    pub fn get_all_quantified(start: Integer, end: Integer) -> QuantifierResult {
        // let integer_range_parameters = IntegerRangeParameters::from_abi(&parameters).unwrap();
        if end < start {
            panic!("invalid start and end");
        }
        QuantifierResult::new(get_range(start, end), true)
    }
}

/// NonnegativeIntegerLessThanQuantifier quantify 0 to upper bound
pub struct NonnegativeIntegerLessThanQuantifier {}

impl Default for NonnegativeIntegerLessThanQuantifier {
    fn default() -> Self {
        NonnegativeIntegerLessThanQuantifier {}
    }
}

impl NonnegativeIntegerLessThanQuantifier {
    pub fn get_all_quantified(upper_bound: Integer) -> QuantifierResult {
        if upper_bound < Integer(0) {
            panic!("upper_bound shouldn't negative value.");
        }
        QuantifierResult::new(get_range(Integer(0), upper_bound), true)
    }
}
