use crate::property_executor::PropertyExecutor;
use crate::types::{InputType, QuantifierResult, QuantifierResultItem};
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
        let preimage = decider.replace(&placeholder).to_integer();
        QuantifierResult::new(
            vec![QuantifierResultItem::H256(static_hash(&preimage.into()))],
            true,
        )
    }
}
