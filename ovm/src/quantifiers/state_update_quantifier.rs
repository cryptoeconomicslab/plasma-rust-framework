use crate::property_executor::PropertyExecutor;
use crate::types::{PropertyInput, QuantifierResult, QuantifierResultItem, StateUpdate};
use plasma_db::traits::kvs::KeyValueStore;

pub struct StateUpdateQuantifier {}

impl Default for StateUpdateQuantifier {
    fn default() -> Self {
        Self {}
    }
}

impl StateUpdateQuantifier {
    pub fn get_all_quantified<KVS>(
        decider: &PropertyExecutor<KVS>,
        inputs: &[PropertyInput],
    ) -> QuantifierResult
    where
        KVS: KeyValueStore,
    {
        let block_number = decider.get_variable(&inputs[0]).to_integer();
        let deposit_contract_address = decider.get_variable(&inputs[1]).to_address();
        let coin_range = decider.get_variable(&inputs[2]).to_range();
        let property = decider.get_variable(&inputs[3]).to_property();
        let state_update =
            StateUpdate::new(block_number, deposit_contract_address, coin_range, property);
        QuantifierResult::new(vec![QuantifierResultItem::StateUpdate(state_update)], true)
    }
}
