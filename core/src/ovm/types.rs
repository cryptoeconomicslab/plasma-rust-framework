use bytes::Bytes;

use ethabi::Token;
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
}

impl From<Property> for Token {
    fn from(property: Property) -> Token {
        Token::Tuple(vec![
            Token::Address(property.decider),
            Token::Bytes(property.input.to_vec()),
        ])
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

impl From<ImplicationProofElement> for Token {
    fn from(element: ImplicationProofElement) -> Token {
        Token::Tuple(vec![
            element.implication.into(),
            Token::Bytes(element.implication_witness.to_vec()),
        ])
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
}
