use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, ImplicationProofElement, NotDeciderInput, Property};
use crate::DecideMixin;
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
        decider: &PropertyExecutor,
        input: &NotDeciderInput,
        _witness: Option<&Bytes>,
    ) -> Result<Decision, Error> {
        let decision = input
            .get_property()
            .decide(decider, Some(input.get_witness()))?;

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
        decider: &PropertyExecutor,
        input: &NotDeciderInput,
    ) -> Result<Decision, Error> {
        Self::decide(decider, input, None)
    }
}
