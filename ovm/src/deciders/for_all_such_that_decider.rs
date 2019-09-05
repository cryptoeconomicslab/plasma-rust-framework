use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{
    Decider, Decision, ForAllSuchThatInput, ImplicationProofElement, Property, QuantifierResult,
};
use crate::DecideMixin;
use plasma_db::traits::kvs::KeyValueStore;

/// ForAllSuchThatDecider decides for all quantified results by PropertyFactory and WitnessFactory
pub struct ForAllSuchThatDecider {}

impl ForAllSuchThatDecider {
    fn get_decision(
        input: &ForAllSuchThatInput,
        false_decision: Decision,
        true_decisions: Vec<Decision>,
        undecided: bool,
    ) -> Result<Decision, Error> {
        if !false_decision.get_outcome() && undecided {
            return Err(Error::from(ErrorKind::Undecided));
        }
        let mut justification = vec![ImplicationProofElement::new(
            Property::ForAllSuchThatDecider(Box::new(input.clone())),
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
    type Input = ForAllSuchThatInput;
    fn decide<T: KeyValueStore>(
        decider: &mut PropertyExecutor<T>,
        input: &ForAllSuchThatInput,
    ) -> Result<Decision, Error> {
        let quantifier_result: QuantifierResult =
            decider.get_all_quantified(input.get_quantifier());

        let mut any_undecided: bool = false;
        let mut false_decision: Decision = Decision::new(false, vec![]);
        let mut true_decisions: Vec<Decision> = vec![];
        for res in quantifier_result.get_results() {
            let prop: Property = input.get_property().clone();
            let _no_cache = false;
            decider.set_replace(input.get_placeholder().clone(), res.clone());
            let decision_result = prop.decide(
                decider,
                // no_cache,
            );

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
            input,
            false_decision,
            true_decisions,
            any_undecided || !quantifier_result.get_all_results_quantified(),
        )
    }
}

/*
#[cfg(test)]
mod tests {
    use super::ForAllSuchThatDecider;
    use crate::db::HashPreimageDb;
    use crate::deciders::preimage_exists_decider::Verifier;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{
        Decider, Decision, ForAllSuchThatInput, Integer, IntegerRangeQuantifierInput,
        PreimageExistsInput, Property, PropertyFactory, Quantifier, QuantifierResultItem, Witness,
    };
    use plasma_db::impls::kvs::CoreDbLevelDbImpl;

    #[test]
    fn test_decide() {
        let input = ForAllSuchThatInput::new(
            Quantifier::IntegerRangeQuantifier(IntegerRangeQuantifierInput::new(5, 20)),
            Some(PropertyFactory::new(Box::new(|item| {
                if let QuantifierResultItem::Integer(number) = item {
                    Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(
                        Verifier::static_hash(&number.into()),
                    )))
                } else {
                    panic!("invalid type of item");
                }
            }))),
        );
        let decider: PropertyExecutor<CoreDbLevelDbImpl> = Default::default();
        let db = HashPreimageDb::new(decider.get_db());
        for i in 5..20 {
            let integer = Integer(i);
            assert!(db
                .store_witness(
                    Verifier::static_hash(&integer.into()),
                    &Witness::Bytes(integer.into())
                )
                .is_ok());
        }
        let decided: Decision = ForAllSuchThatDecider::decide(&decider, &input).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }
}
*/
