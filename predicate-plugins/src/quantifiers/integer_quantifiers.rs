use bytes::Bytes;
use ethabi::{ParamType, Token};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{Error, ErrorKind};
use plasma_core::ovm::{Quantifier, QuantifierResult};

#[derive(Clone, Debug)]
pub struct Integer {
    n: u64,
}

impl Integer {
    pub fn new(n: u64) -> Self {
        Integer { n }
    }
}

impl Encodable for Integer {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![Token::Uint(self.n.into())]
    }
}

fn get_range(start: u64, end: u64) -> Vec<Bytes> {
    (start..end)
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

pub struct IntegerRangeQuantifier {}

impl Quantifier for IntegerRangeQuantifier {
    fn get_all_quantified(&self, parameters: Bytes) -> QuantifierResult {
        let integer_range_parameters = IntegerRangeParameters::from_abi(&parameters).unwrap();
        if integer_range_parameters.get_end() < integer_range_parameters.get_start() {
            panic!("invalid start and end");
        }
        QuantifierResult::new(
            get_range(
                integer_range_parameters.get_start(),
                integer_range_parameters.get_end(),
            ),
            true,
        )
    }
}
