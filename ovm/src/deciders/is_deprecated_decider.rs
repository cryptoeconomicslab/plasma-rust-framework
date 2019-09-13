use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, InputType, Property};
use crate::DecideMixin;
use plasma_db::traits::kvs::KeyValueStore;

pub struct IsDeprecatedDecider {}

impl Decider for IsDeprecatedDecider {
    fn decide<T: KeyValueStore>(
        decider: &mut PropertyExecutor<T>,
        inputs: &Vec<InputType>,
    ) -> Result<Decision, Error> {
        let state_update = decider.get_variable(&inputs[0]).to_state_update();
        let property = state_update.get_property();
        println!("IsDeprecatedDecider {:?}", state_update);
        let decided = property.decide(decider);
        assert!(decided.is_ok());
        Ok(Decision::new(true, vec![]))
    }
}
