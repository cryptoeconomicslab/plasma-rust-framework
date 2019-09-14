use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, ImplicationProofElement, InputType, QuantifierResult};
use crate::{DecideMixin, DeciderManager};
use plasma_db::traits::kvs::KeyValueStore;

/// ForAllSuchThatDecider decides for all quantified results by PropertyFactory and WitnessFactory
pub struct ForAllSuchThatDecider {}

impl ForAllSuchThatDecider {
    fn get_decision(
        inputs: &[InputType],
        false_decision: Decision,
        true_decisions: Vec<Decision>,
        undecided: bool,
    ) -> Result<Decision, Error> {
        if !false_decision.get_outcome() && undecided {
            return Err(Error::from(ErrorKind::Undecided));
        }
        let mut justification = vec![ImplicationProofElement::new(
            DeciderManager::for_all_such_that_decider_raw(inputs),
            None,
        )];
        if false_decision.get_outcome() {
            justification.extend(false_decision.get_implication_proof().clone())
        } else {
            for decision in true_decisions {
                justification.extend(decision.get_implication_proof().clone())
            }
        }

        Ok(Decision::new(!false_decision.get_outcome(), justification))
    }
}

impl Default for ForAllSuchThatDecider {
    fn default() -> Self {
        ForAllSuchThatDecider {}
    }
}

impl Decider for ForAllSuchThatDecider {
    fn decide<T: KeyValueStore>(
        decider: &mut PropertyExecutor<T>,
        inputs: &[InputType],
    ) -> Result<Decision, Error> {
        let quantifier = decider.get_variable(&inputs[0]).to_property();
        let placeholder = decider.get_variable(&inputs[1]).to_bytes();
        let property = decider.get_variable(&inputs[2]).to_property();

        let quantifier_result: QuantifierResult = decider.get_all_quantified(&quantifier);

        let mut any_undecided: bool = false;
        let mut false_decision: Decision = Decision::new(false, vec![]);
        let mut true_decisions: Vec<Decision> = vec![];
        for res in quantifier_result.get_results() {
            decider.set_variable(placeholder.clone(), res.clone());
            let decision_result = property.decide(decider);
            if let Ok(decision) = decision_result {
                if !decision.get_outcome() {
                    false_decision = decision;
                    break;
                }
                true_decisions.push(decision)
            } else {
                any_undecided = true;
            }
        }

        Self::get_decision(
            inputs,
            false_decision,
            true_decisions,
            any_undecided || !quantifier_result.get_all_results_quantified(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ForAllSuchThatDecider;
    use crate::db::HashPreimageDb;
    use crate::deciders::preimage_exists_decider::Verifier;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{Decider, Decision, InputType, Integer};
    use crate::DeciderManager;
    use bytes::Bytes;
    use plasma_core::data_structure::Range;
    use plasma_db::impls::kvs::CoreDbMemoryImpl;

    #[test]
    fn test_decide() {
        let property = DeciderManager::for_all_such_that_decider(
            DeciderManager::q_range(vec![InputType::ConstantRange(Range::new(5, 20))]),
            Bytes::from("n"),
            DeciderManager::for_all_such_that_decider(
                DeciderManager::q_hash(vec![InputType::Placeholder(Bytes::from("n"))]),
                Bytes::from("h"),
                DeciderManager::preimage_exists_decider(vec![InputType::Placeholder(Bytes::from(
                    "h",
                ))]),
            ),
        );
        let mut decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        let db = HashPreimageDb::new(decider.get_db());
        for i in 5..20 {
            let integer = Integer(i);
            assert!(db
                .store_witness(Verifier::static_hash(&integer.into()), &integer.into())
                .is_ok());
        }
        let decided: Decision =
            ForAllSuchThatDecider::decide(&mut decider, &property.inputs).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }
}
