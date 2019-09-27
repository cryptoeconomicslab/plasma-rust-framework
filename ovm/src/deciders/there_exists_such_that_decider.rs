use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, ImplicationProofElement, PropertyInput, QuantifierResult};
use crate::{DecideMixin, DeciderManager};
use plasma_db::traits::kvs::KeyValueStore;

pub struct ThereExistsSuchThatDecider {}

impl ThereExistsSuchThatDecider {
    fn get_decision(
        inputs: &[PropertyInput],
        true_decisions: Decision,
        undecided: bool,
    ) -> Result<Decision, Error> {
        if undecided {
            return Err(Error::from(ErrorKind::Undecided));
        }
        let mut justification = vec![ImplicationProofElement::new(
            DeciderManager::there_exists_such_that(inputs.to_vec()),
            None,
        )];
        if true_decisions.get_outcome() {
            justification.extend(true_decisions.get_implication_proof().clone())
        }
        Ok(Decision::new(true_decisions.get_outcome(), justification))
    }
}

impl Default for ThereExistsSuchThatDecider {
    fn default() -> Self {
        Self {}
    }
}

impl Decider for ThereExistsSuchThatDecider {
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        inputs: &[PropertyInput],
    ) -> Result<Decision, Error> {
        let quantifier = decider.get_variable(&inputs[0]).to_property();
        let placeholder = decider.get_variable(&inputs[1]).to_bytes();
        let property = decider.get_variable(&inputs[2]).to_property();

        let quantifier_result: QuantifierResult = decider.get_all_quantified(&quantifier);

        let mut true_decision: Decision = Decision::new(false, vec![]);
        for res in quantifier_result.get_results() {
            decider.set_variable(placeholder.clone(), res.clone());
            let decision_result = property.decide(decider);
            if let Ok(decision) = decision_result {
                if decision.get_outcome() {
                    true_decision = decision;
                    break;
                }
            }
        }

        Self::get_decision(
            inputs,
            true_decision,
            !quantifier_result.get_all_results_quantified(),
        )
    }
}
