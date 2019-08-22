use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, HasLowerNonceInput, ImplicationProofElement, Property};
use plasma_db::traits::kvs::KeyValueStore;

pub struct HasLowerNonceDecider {}

impl HasLowerNonceDecider {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for HasLowerNonceDecider {
    fn default() -> Self {
        Self {}
    }
}

impl Decider for HasLowerNonceDecider {
    type Input = HasLowerNonceInput;
    fn decide<T: KeyValueStore>(
        _decider: &PropertyExecutor<T>,
        input: &HasLowerNonceInput,
    ) -> Result<Decision, Error> {
        if input.get_message().nonce < input.get_nonce() {
            Ok(Decision::new(
                true,
                vec![ImplicationProofElement::new(
                    Property::HasLowerNonceDecider(input.clone()),
                    None,
                )],
            ))
        } else {
            Ok(Decision::new(false, vec![]))
        }
    }
}
