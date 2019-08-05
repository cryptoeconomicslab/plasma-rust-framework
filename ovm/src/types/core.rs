use super::inputs::{AndDeciderInput, ForAllSuchThatInput, NotDeciderInput, PreimageExistsInput};
use crate::error::Error;
use crate::property_executer::PropertyExecuter;
use bytes::Bytes;
use ethabi::Token;
use ethereum_types::{Address, H256};
use plasma_core::data_structure::abi::Encodable;
use std::sync::Arc;

pub type DeciderId = Address;
pub type QuantifierId = Address;
pub trait Input {}
pub trait Witness {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub struct Integer(pub u64);

impl Integer {
    pub fn new(n: u64) -> Self {
        Integer(n)
    }
}

impl Encodable for Integer {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![Token::Uint(self.0.into())]
    }
}

/// The property which will be decided by Decider
#[derive(Clone, Debug)]
pub enum Property {
    // left, left_witness, right, right_witness
    AndDecider(Box<AndDeciderInput>),
    // property, witness
    NotDecider(Box<NotDeciderInput>),
    // quantifier, quantifier_parameters, property_factory, witness_factory?
    ForAllSuchThatDecider(Box<ForAllSuchThatInput>),
    // hash
    PreimageExistsDecider(Box<PreimageExistsInput>),
    // message, public_key
    SignedByDecider(Bytes, Address),
    // channelId, nonce, participant
    ChannelUpdateSignatureExistsDecider(H256, Integer, Address),
}

#[derive(Clone, Debug)]
pub enum Quantifier {
    // start to end
    IntegerRangeQuantifier(Integer, Integer),
    // 0 to upperBound
    NonnegativeIntegerLessThanQuantifier(Integer),
    // blocknumber, start and end
    IncludedCoinRangeQuantifier(Integer, Integer, Integer),
}

impl Property {
    pub fn get_decider_id(&self) -> DeciderId {
        match self {
            Property::AndDecider(_) => Address::zero(),
            _ => Address::zero(),
        }
    }
}

impl Encodable for Property {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        match self {
            Property::AndDecider(input) => vec![
                Token::Address(self.get_decider_id()),
                Token::Tuple(input.get_left().to_tuple()),
                Token::Bytes(input.get_left_witness().to_vec()),
                Token::Tuple(input.get_right().to_tuple()),
                Token::Bytes(input.get_right_witness().to_vec()),
            ],
            _ => vec![Token::Address(self.get_decider_id())],
        }
    }
}

/*
We don't define decoder for Property now
impl Decodable for Property {
    type Ok = Property;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let decider = tuple[0].clone().to_address();
        let input = tuple[1].clone().to_bytes();
        if let (Some(decider), Some(input)) = (decider, input) {
            Ok(Property::new(decider, Bytes::from(input)))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded = ethabi::decode(&[ParamType::Address, ParamType::Bytes], data)
            .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}
*/

/// Implication proof element has the property which is decided by Decider
#[derive(Clone, Debug)]
pub struct ImplicationProofElement {
    implication: Property,
    implication_witness: Option<Bytes>,
}

impl ImplicationProofElement {
    pub fn new(implication: Property, implication_witness: Option<Bytes>) -> Self {
        ImplicationProofElement {
            implication,
            implication_witness,
        }
    }
}

/// Decision made by Decider
#[derive(Clone, Debug)]
pub struct Decision {
    outcome: bool,
    implication_proof: Vec<ImplicationProofElement>,
}

impl Decision {
    pub fn new(outcome: bool, implication_proof: Vec<ImplicationProofElement>) -> Self {
        Decision {
            outcome,
            implication_proof,
        }
    }
    pub fn get_outcome(&self) -> bool {
        self.outcome
    }
    pub fn get_implication_proof(&self) -> &Vec<ImplicationProofElement> {
        &self.implication_proof
    }
}

#[derive(Clone)]
pub struct PropertyFactory(Arc<Fn(Bytes) -> Property>);

impl PropertyFactory {
    pub fn new(handler: Box<Fn(Bytes) -> Property>) -> Self {
        PropertyFactory(Arc::new(handler))
    }
    pub fn call(&self, bytes: Bytes) -> Property {
        self.0(bytes)
    }
}

impl std::fmt::Debug for PropertyFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PropertyFactory")
    }
}

#[derive(Clone)]
pub struct WitnessFactory(Arc<Fn(Bytes) -> Bytes>);

impl WitnessFactory {
    pub fn new(handler: Box<Fn(Bytes) -> Bytes>) -> Self {
        WitnessFactory(Arc::new(handler))
    }
    pub fn call(&self, bytes: Bytes) -> Bytes {
        self.0(bytes)
    }
}

impl std::fmt::Debug for WitnessFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WitnessFactory")
    }
}

pub trait Decider {
    type Input;
    fn decide(
        decider: &PropertyExecuter,
        input: &Self::Input,
        witness: Option<&Bytes>,
    ) -> Result<Decision, Error>;
    fn check_decision(decider: &PropertyExecuter, input: &Self::Input) -> Result<Decision, Error>;
}

pub struct QuantifierResult {
    results: Vec<Bytes>,
    all_results_quantified: bool,
}

impl QuantifierResult {
    pub fn new(results: Vec<Bytes>, all_results_quantified: bool) -> Self {
        QuantifierResult {
            results,
            all_results_quantified,
        }
    }
    pub fn get_results(&self) -> &Vec<Bytes> {
        &self.results
    }
    pub fn get_all_results_quantified(&self) -> bool {
        self.all_results_quantified
    }
}
