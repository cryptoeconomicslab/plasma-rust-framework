use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, ImplicationProofElement, InputType};
use crate::{DecideMixin, DeciderManager};
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
    fn decide<T: KeyValueStore>(
        decider: &mut PropertyExecutor<T>,
        inputs: &Vec<InputType>,
    ) -> Result<Decision, Error> {
        let property = decider.get_variable(&inputs[0]).to_property();
        let decision = property.decide(decider)?;

        Ok(Decision::new(
            !decision.get_outcome(),
            [
                &vec![ImplicationProofElement::new(
                    DeciderManager::not_decider(property),
                    None,
                )][..],
                &decision.get_implication_proof()[..],
            ]
            .concat(),
        ))
    }
}
