use crate::property_executor::PropertyExecutor;
use crate::types::{InputType, Integer, QuantifierResult, QuantifierResultItem};
use crate::utils::static_hash;
use plasma_db::traits::kvs::KeyValueStore;

pub struct HashQuantifier {}

impl Default for HashQuantifier {
    fn default() -> Self {
        Self {}
    }
}

impl HashQuantifier {
    pub fn get_all_quantified<KVS: KeyValueStore>(
        decider: &PropertyExecutor<KVS>,
        placeholder: &InputType,
    ) -> QuantifierResult {
        if let QuantifierResultItem::Integer(preimage) = decider.replace(&placeholder) {
            if preimage < Integer(0) {
                panic!("preimage shouldn't negative value.");
            }
            QuantifierResult::new(
                vec![QuantifierResultItem::H256(static_hash(&preimage.into()))],
                true,
            )
        } else {
            panic!("invalid input")
        }
    }
}
