use crate::error::Error;
use crate::property_executer::PropertyExecuter;
use crate::types::{Decider, Decision, ImplicationProofElement, NotDeciderInput, Property};
use bytes::Bytes;

pub struct NotDecider {}

impl NotDecider {
    pub fn new() -> Self {
        NotDecider {}
    }
}

impl Default for NotDecider {
    fn default() -> Self {
        NotDecider {}
    }
}

impl Decider for NotDecider {
    type Input = NotDeciderInput;
    fn decide(
        decider: &PropertyExecuter,
        input: &NotDeciderInput,
        _witness: Option<&Bytes>,
    ) -> Result<Decision, Error> {
        let decision = decider.decide(input.get_property(), Some(input.get_witness()))?;

        Ok(Decision::new(
            !decision.get_outcome(),
            [
                &vec![ImplicationProofElement::new(
                    Property::NotDecider(Box::new(input.clone())),
                    None,
                )][..],
                &decision.get_implication_proof()[..],
            ]
            .concat(),
        ))
    }

    fn check_decision(
        decider: &PropertyExecuter,
        input: &NotDeciderInput,
    ) -> Result<Decision, Error> {
        Self::decide(decider, input, None)
    }
}
