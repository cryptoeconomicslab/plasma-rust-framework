use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, ImplicationProofElement, PropertyInput};
use crate::DeciderManager;
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
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        inputs: &[PropertyInput],
    ) -> Result<Decision, Error> {
        let message = decider.get_variable(&inputs[0]).to_message();
        let nonce = decider.get_variable(&inputs[1]).to_integer();
        if message.nonce < nonce {
            Ok(Decision::new(
                true,
                vec![ImplicationProofElement::new(
                    DeciderManager::has_lower_nonce_decider(inputs.to_vec()),
                    None,
                )],
            ))
        } else {
            Ok(Decision::new(false, vec![]))
        }
    }
}
