use crate::property_executor::PropertyExecutor;
use crate::types::QuantifierResult;
use bytes::Bytes;
use ethereum_types::Address;
use plasma_core::data_structure::abi::Encodable;
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
        let messages = decider
            .get_message_db()
            .get_messages_signed_by(signed_by, None, None);
        QuantifierResult::new(
            messages.iter().map(|m| Bytes::from(m.to_abi())).collect(),
            true,
        )
    }
}
