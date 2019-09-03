use crate::db::SignedByDb;
use crate::property_executor::PropertyExecutor;
use crate::types::{InputType, QuantifierResult, QuantifierResultItem};
use plasma_db::traits::kvs::KeyValueStore;

pub struct SignedByQuantifier {}

impl Default for SignedByQuantifier {
    fn default() -> Self {
        SignedByQuantifier {}
    }
}

impl SignedByQuantifier {
    pub fn get_all_quantified<KVS>(
        decider: &PropertyExecutor<KVS>,
        signed_by: &InputType,
    ) -> QuantifierResult
    where
        KVS: KeyValueStore,
    {
        let signed_by = decider.replace(signed_by).to_address();
        let message_db = SignedByDb::new(decider.get_db());
        let messages = message_db.get_all_signed_by(signed_by);
        QuantifierResult::new(
            messages
                .iter()
                .map(|m| QuantifierResultItem::Bytes(m.message.clone()))
                .collect(),
            true,
        )
    }
}
