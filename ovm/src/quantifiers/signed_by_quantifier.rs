use crate::db::SignedByDb;
use crate::property_executor::PropertyExecutor;
use crate::types::{QuantifierResult, QuantifierResultItem};
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
        signed_by: Address,
    ) -> QuantifierResult
    where
        KVS: KeyValueStore,
    {
        let db: SignedByDb<KVS> = SignedByDb::new(decider.get_db());
        let messages = db.get_all_signed_by(signed_by);
        QuantifierResult::new(
            messages
                .iter()
                .map(|m| QuantifierResultItem::Bytes(m.message.clone()))
                .collect(),
            true,
        )
    }
}
