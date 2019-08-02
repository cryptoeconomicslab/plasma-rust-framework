use crate::ovm::Decider;
use bytes::Bytes;

pub trait Input {}
pub trait Witness {}

/// The property which will be decided by Decider
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Property<D: Decider> {
    decider: D,
    input: Bytes,
}

impl<D> Property<D>
where
    D: Decider,
{
    pub fn new(decider: D, input: Bytes) -> Self {
        Property { decider, input }
    }
}

/// Implication proof element has the property which is decided by Decider
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImplicationProofElement<D: Decider> {
    implication: Property<D>,
    implication_witness: Bytes,
}

impl<D> ImplicationProofElement<D>
where
    D: Decider,
{
    pub fn new(implication: Property<D>, implication_witness: Bytes) -> Self {
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
pub struct Decision<D: Decider> {
    outcome: DecisionStatus,
    implication_proof: Vec<ImplicationProofElement<D>>,
}

impl<D> Decision<D>
where
    D: Decider,
{
    pub fn new(
        outcome: DecisionStatus,
        implication_proof: Vec<ImplicationProofElement<D>>,
    ) -> Self {
        Decision {
            outcome,
            implication_proof,
        }
    }
    pub fn get_outcome(&self) -> &DecisionStatus {
        &self.outcome
    }
}
