use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, PropertyInput};
use crate::DecideMixin;
use plasma_db::traits::kvs::KeyValueStore;

pub struct IsDeprecatedDecider {}

impl Decider for IsDeprecatedDecider {
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        inputs: &[PropertyInput],
    ) -> Result<Decision, Error> {
        let state_update = decider.get_variable(&inputs[0]).to_state_update();
        let property = state_update.get_property();
        let decided = property.decide(decider);
        assert!(decided.is_ok());
        Ok(Decision::new(true, vec![]))
    }
}
