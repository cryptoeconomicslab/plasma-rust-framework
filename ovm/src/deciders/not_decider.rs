use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, ImplicationProofElement, NotDeciderInput, Property};
use crate::DecideMixin;
use plasma_db::traits::kvs::KeyValueStore;

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
    fn decide<T: KeyValueStore>(
        decider: &mut PropertyExecutor<T>,
        input: &NotDeciderInput,
    ) -> Result<Decision, Error> {
        let decision = input.get_property().decide(decider)?;

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
}
