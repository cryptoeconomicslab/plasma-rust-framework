use crate::error::Error;
use crate::property_executor::DecideMixin;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, OrDeciderInput, Witness};
use plasma_db::traits::kvs::KeyValueStore;

pub struct OrDecider {}

impl OrDecider {
    pub fn new() -> Self {
        OrDecider {}
    }
}

impl Default for OrDecider {
    fn default() -> Self {
        OrDecider {}
    }
}

impl Decider for OrDecider {
    type Input = OrDeciderInput;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &OrDeciderInput,
        _witness: Option<Witness>,
    ) -> Result<Decision, Error> {
        let left_decision = input
            .get_left()
            .decide(decider, Some(input.get_left_witness().clone()))?;
        let right_decision = input
            .get_right()
            .decide(decider, Some(input.get_right_witness().clone()))?;
        if left_decision.get_outcome() {
            return Ok(left_decision);
        }
        if right_decision.get_outcome() {
            return Ok(right_decision);
        }
        Ok(Decision::new(
            false,
            [
                &left_decision.get_implication_proof()[..],
                &right_decision.get_implication_proof()[..],
            ]
            .concat(),
        ))
    }

    fn check_decision<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &OrDeciderInput,
    ) -> Result<Decision, Error> {
        Self::decide(decider, input, None)
    }
}

#[cfg(test)]
mod tests {
    use super::OrDecider;
    use crate::deciders::preimage_exists_decider::Verifier;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{
        Decider, Decision, NotDeciderInput, OrDeciderInput, PreimageExistsInput, Property, Witness,
    };
    use bytes::Bytes;
    use plasma_db::impls::kvs::CoreDbLevelDbImpl;

    #[test]
    fn test_decide() {
        let left = Property::PreimageExistsDecider(PreimageExistsInput::new(
            Verifier::static_hash(&Bytes::from("left")),
        ));
        let left_witness = Witness::Bytes(Bytes::from("left"));
        let right = Property::PreimageExistsDecider(PreimageExistsInput::new(
            Verifier::static_hash(&Bytes::from("right")),
        ));
        let right_witness = Witness::Bytes(Bytes::from("right"));
        let input = OrDeciderInput::new(
            left,
            left_witness,
            Property::NotDecider(Box::new(NotDeciderInput::new(right, right_witness))),
            Witness::Bytes(Bytes::from("not")),
        );
        let or_decider = Property::OrDecider(Box::new(input.clone()));
        let decider: PropertyExecutor<CoreDbLevelDbImpl> = Default::default();
        let decided: Decision = decider.decide(&or_decider, None).unwrap();
        assert_eq!(decided.get_outcome(), true);
        let status = OrDecider::check_decision(&decider, &input).unwrap();
        assert_eq!(status.get_outcome(), true);
    }

}
