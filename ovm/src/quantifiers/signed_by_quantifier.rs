use crate::db::SignedByDb;
use crate::property_executor::PropertyExecutor;
use crate::types::{Placeholder, QuantifierResult, QuantifierResultItem};
use ethereum_types::Address;
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
        signed_by: &Placeholder,
    ) -> QuantifierResult
    where
        KVS: KeyValueStore,
    {
        if let QuantifierResultItem::Address(signed_by) = decider.replace(&signed_by) {
            let message_db = SignedByDb::new(decider.get_db());
            let messages = message_db.get_all_signed_by(*signed_by);
            QuantifierResult::new(
                messages
                    .iter()
                    .map(|m| QuantifierResultItem::Bytes(m.message.clone()))
                    .collect(),
                true,
            )
        } else {
            panic!("invalid input")
        }
    }
}
