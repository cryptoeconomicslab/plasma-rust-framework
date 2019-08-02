use bytes::Bytes;
use ethereum_types::Address;

type DeciderId = Address;
pub trait Input {}
pub trait Witness {}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionStatus {
    Decided(bool),
    Undecided,
}

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
