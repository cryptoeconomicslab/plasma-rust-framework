use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, IsDeprecatedInput};
use plasma_db::traits::kvs::KeyValueStore;

pub struct IsDeprecatedDecider {}

impl Default for IsDeprecatedDecider {
    fn default() -> Self {
        IsDeprecatedDecider {}
    }
}

impl Decider for IsDeprecatedDecider {
    type Input = IsDeprecatedInput;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &IsDeprecatedInput,
    ) -> Result<Decision, Error> {
        decider.decide(input.plasma_data_block.get_property())
    }
}
