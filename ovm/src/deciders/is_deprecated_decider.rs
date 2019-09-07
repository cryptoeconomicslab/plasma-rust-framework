use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, IsDeprecatedDeciderInput, Property, StateUpdate};
use crate::DecideMixin;
use plasma_db::traits::kvs::KeyValueStore;

pub struct IsDeprecatedDecider {}

impl Decider for IsDeprecatedDecider {
    type Input = IsDeprecatedDeciderInput;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &IsDeprecatedDeciderInput,
    ) -> Result<Decision, Error> {
        let state_update = input.get_state_update();
        let address = state_update.get_property_address();
//        let property = Property::get_generalized_plasma_property(address, state_update)
//        let decided = property.decide(decider)
        Ok(Decision::new(true, vec![]))
    }
}
