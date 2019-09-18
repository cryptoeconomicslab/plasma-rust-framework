use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, PropertyInput};
use crate::DecideMixin;
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
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        inputs: &[PropertyInput],
    ) -> Result<Decision, Error> {
        let left = decider.get_variable(&inputs[0]).to_property();
        let right = decider.get_variable(&inputs[1]).to_property();
        let left_decision = left.decide(decider);
        let right_decision = right.decide(decider);
        if let Ok(left_decision) = &left_decision {
            if left_decision.get_outcome() {
                return Ok(left_decision.clone());
            }
        }
        if let Ok(right_decision) = &right_decision {
            if right_decision.get_outcome() {
                return Ok(right_decision.clone());
            }
        }
        if left_decision.is_err() || right_decision.is_err() {
            return Err(Error::from(ErrorKind::CannotDecide));
        }
        Ok(Decision::new(
            false,
            [
                &left_decision.unwrap().clone().get_implication_proof()[..],
                &right_decision.unwrap().clone().get_implication_proof()[..],
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
        let left_hash = Verifier::static_hash(&Bytes::from("left"));
        let right_hash = Verifier::static_hash(&Bytes::from("right"));
        let left = DeciderManager::preimage_exists_decider(vec![PropertyInput::ConstantH256(
            Verifier::static_hash(&Bytes::from("left")),
        )]);
        let left_witness = Bytes::from("left");
        let right = DeciderManager::preimage_exists_decider(vec![PropertyInput::ConstantH256(
            Verifier::static_hash(&Bytes::from("right")),
        )]);
        let right_witness = Bytes::from("right");
        let or_decider = DeciderManager::or_decider(left, DeciderManager::not_decider(right));
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        let db = HashPreimageDb::new(decider.get_db());
        assert!(db.store_witness(left_hash, &left_witness).is_ok());
        assert!(db.store_witness(right_hash, &right_witness).is_ok());
        let decided: Decision = decider.decide(&or_decider).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }
}
