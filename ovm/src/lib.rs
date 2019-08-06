pub mod deciders;
pub mod error;
pub mod property_executer;
pub mod quantifiers;
pub mod types;

#[cfg(test)]
mod tests {

    use crate::deciders::preimage_exists_decider::Verifier;
    use crate::property_executer::PropertyExecuter;
    use crate::types::{
        Decision, ForAllSuchThatInput, Integer, PreimageExistsInput, Property, PropertyFactory,
        Quantifier, WitnessFactory,
    };
    use bytes::Bytes;

    #[test]
    fn test_decide_range_and_preimage() {
        let property = Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
            Quantifier::IntegerRangeQuantifier(Integer(0), Integer(10)),
            PropertyFactory::new(Box::new(|bytes| {
                Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(
                    Verifier::static_hash(&bytes),
                )))
            })),
            WitnessFactory::new(Box::new(|bytes| bytes.clone())),
        )));
        let decider: PropertyExecuter = Default::default();
        let decided: Decision = decider.decide(&property, None).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }

    #[test]
    fn test_fail_to_decide_range_and_preimage() {
        let property = Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
            Quantifier::IntegerRangeQuantifier(Integer(0), Integer(10)),
            PropertyFactory::new(Box::new(|bytes| {
                Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(
                    Verifier::static_hash(&bytes),
                )))
            })),
            WitnessFactory::new(Box::new(|_bytes| Bytes::from(&b"aaa"[..]))),
        )));
        let decider: PropertyExecuter = Default::default();
        let decided_result = decider.decide(&property, None);
        assert_eq!(decided_result.is_ok(), false);
    }

    #[test]
    fn test_decide_less_than_and_preimage() {
        let property = Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
            Quantifier::NonnegativeIntegerLessThanQuantifier(Integer(10)),
            PropertyFactory::new(Box::new(|bytes| {
                Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(
                    Verifier::static_hash(&bytes),
                )))
            })),
            WitnessFactory::new(Box::new(|bytes| bytes.clone())),
        )));
        let decider: PropertyExecuter = Default::default();
        let decided: Decision = decider.decide(&property, None).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }

}
