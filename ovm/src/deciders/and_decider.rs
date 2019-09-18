use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, PropertyInput};
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
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        inputs: &[PropertyInput],
    ) -> Result<Decision, Error> {
        let left = decider.get_variable(&inputs[0]).to_property();
        let right = decider.get_variable(&inputs[1]).to_property();
        let left_decision = left.decide(decider)?;
        let right_decision = right.decide(decider)?;
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
    use crate::types::{Decision, PropertyInput};
    use crate::DeciderManager;
    use bytes::Bytes;
    use plasma_db::impls::kvs::CoreDbMemoryImpl;

    #[test]
    fn test_decide() {
        let left_preimage = Bytes::from("left");
        let left_hash = Verifier::static_hash(&left_preimage);
        let right_preimage = Bytes::from("right");
        let right_hash = Verifier::static_hash(&right_preimage);
        let left =
            DeciderManager::preimage_exists_decider(vec![PropertyInput::ConstantH256(left_hash)]);
        let right =
            DeciderManager::preimage_exists_decider(vec![PropertyInput::ConstantH256(right_hash)]);
        let and_decider = DeciderManager::and_decider(left, right);
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        let db = HashPreimageDb::new(decider.get_db());
        assert!(db.store_witness(left_hash, &left_preimage).is_ok());
        assert!(db.store_witness(right_hash, &right_preimage).is_ok());
        let decided: Decision = decider.decide(&and_decider).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }
}
