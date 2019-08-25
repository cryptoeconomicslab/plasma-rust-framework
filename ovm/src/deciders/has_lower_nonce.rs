use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{
    Decider, Decision, HasLowerNonceInput, ImplicationProofElement, Property, QuantifierResultItem,
};
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
        decider: &mut PropertyExecutor<T>,
        input: &HasLowerNonceInput,
    ) -> Result<Decision, Error> {
        if let (QuantifierResultItem::Message(message), QuantifierResultItem::Integer(nonce)) = (
            decider.replace(input.get_message()),
            decider.replace(input.get_nonce()),
        ) {
            if message.nonce < *nonce {
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
        } else {
            panic!("invalid input");
        }
    }
}
