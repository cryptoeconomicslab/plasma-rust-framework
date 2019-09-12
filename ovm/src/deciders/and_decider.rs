use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{AndDeciderInput, Decider, Decision};
use crate::DecideMixin;
use plasma_db::traits::kvs::KeyValueStore;

pub struct AndDecider {}

impl AndDecider {
    pub fn new() -> Self {
        AndDecider {}
    }
}

impl Default for AndDecider {
    fn default() -> Self {
        AndDecider {}
    }
}

impl Decider for AndDecider {
    type Input = AndDeciderInput;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &AndDeciderInput,
    ) -> Result<Decision, Error> {
        let left_decision = input.get_left().decide(decider)?;
        let right_decision = input.get_right().decide(decider)?;
        if !left_decision.get_outcome() {
            return Ok(left_decision);
        }
        if !right_decision.get_outcome() {
            return Ok(right_decision);
        }
        Ok(Decision::new(
            true,
            [
                &left_decision.get_implication_proof()[..],
                &right_decision.get_implication_proof()[..],
            ]
            .concat(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::db::HashPreimageDb;
    use crate::deciders::preimage_exists_decider::Verifier;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{AndDeciderInput, Decision, PreimageExistsInput, Property};
    use bytes::Bytes;
    use plasma_db::impls::kvs::CoreDbMemoryImpl;

    #[test]
    fn test_decide() {
        let left_preimage = Bytes::from("left");
        let left_hash = Verifier::static_hash(&left_preimage);
        let right_preimage = Bytes::from("right");
        let right_hash = Verifier::static_hash(&right_preimage);
        let left = Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(left_hash)));
        let right = Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(right_hash)));
        let input = AndDeciderInput::new(left, right);
        let and_decider = Property::AndDecider(Box::new(input.clone()));
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        let db = HashPreimageDb::new(decider.get_db());
        assert!(db.store_witness(left_hash, &left_preimage).is_ok());
        assert!(db.store_witness(right_hash, &right_preimage).is_ok());
        let decided: Decision = decider.decide(&and_decider).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }
}
