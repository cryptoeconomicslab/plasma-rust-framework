use bytes::Bytes;
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
}
