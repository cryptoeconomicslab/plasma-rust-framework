use crate::data_structure::abi::{Decodable, Encodable};
use crate::data_structure::error::{Error, ErrorKind};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;

pub type DeciderId = Address;
pub trait Input {}
pub trait Witness {}

/// The property which will be decided by Decider
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Property {
    decider: DeciderId,
    input: Bytes,
}

impl Property {
    pub fn new(decider: DeciderId, input: Bytes) -> Self {
        Property { decider, input }
    }
    pub fn get_decider_id(&self) -> DeciderId {
        self.decider
    }
    pub fn get_input(&self) -> &Bytes {
        &self.input
    }
}

impl Encodable for Property {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Address(self.decider),
            Token::Bytes(self.input.to_vec()),
        ]
    }
}

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

/// Implication proof element has the property which is decided by Decider
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImplicationProofElement {
    implication: Property,
    implication_witness: Bytes,
}

impl ImplicationProofElement {
    pub fn new(implication: Property, implication_witness: Bytes) -> Self {
        ImplicationProofElement {
            implication,
            implication_witness,
        }
    }
}

/// Decision status has 3 state, decided as true or false, or undecided.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionStatus {
    Decided(bool),
    Undecided,
}

/// Decision made by Decider
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decision {
    outcome: DecisionStatus,
    implication_proof: Vec<ImplicationProofElement>,
}

impl Decision {
    pub fn new(outcome: DecisionStatus, implication_proof: Vec<ImplicationProofElement>) -> Self {
        Decision {
            outcome,
            implication_proof,
        }
    }
    pub fn get_outcome(&self) -> &DecisionStatus {
        &self.outcome
    }
    pub fn get_implication_proof(&self) -> &Vec<ImplicationProofElement> {
        &self.implication_proof
    }
}
