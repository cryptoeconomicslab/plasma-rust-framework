use crate::db::TransactionDb;
use crate::property_executor::PropertyExecutor;
use crate::types::{PropertyInput, QuantifierResult, QuantifierResultItem};
use bytes::Bytes;
use plasma_core::data_structure::abi::Encodable;
use plasma_db::traits::kvs::KeyValueStore;

pub struct TxQuantifier {}

impl Default for TxQuantifier {
    fn default() -> Self {
        Self {}
    }
}

impl TxQuantifier {
    pub fn get_all_quantified<KVS>(
        decider: &PropertyExecutor<KVS>,
        inputs: &[PropertyInput],
    ) -> QuantifierResult
    where
        KVS: KeyValueStore,
    {
        let state_update = decider.get_variable(&inputs[0]).to_state_update();
        let db: TransactionDb<KVS> = TransactionDb::new(decider.get_range_db());
        let txs_result =
            db.get_transactions(state_update.get_block_number().0, state_update.get_range());
        if let Ok(txs) = txs_result {
            QuantifierResult::new(
                txs.iter()
                    .filter(move |tx| state_update.get_range().is_subrange(&tx.get_range()))
                    .map(|tx| QuantifierResultItem::Bytes(Bytes::from(tx.to_body_abi())))
                    .collect(),
                true,
            )
        } else {
            QuantifierResult::new(vec![], false)
        }
    }
}
