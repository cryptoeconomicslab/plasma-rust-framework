use crate::property_executor::PropertyExecutor;
use crate::statements::plasma::*;
use crate::types::{Property, PropertyInput, QuantifierResult, QuantifierResultItem};
use plasma_db::traits::kvs::KeyValueStore;

/// PropertyQuantifier is quantifier which calls Property Factory.
pub struct PropertyQuantifier {}

impl Default for PropertyQuantifier {
    fn default() -> Self {
        Self {}
    }
}

impl PropertyQuantifier {
    pub fn get_all_quantified<KVS>(
        decider: &PropertyExecutor<KVS>,
        inputs: &[PropertyInput],
    ) -> QuantifierResult
    where
        KVS: KeyValueStore,
    {
        // In smart contract side, quantifier should call Property Factory contract.println!
        // So, First item of inputs should be address of Property Factpry contract.
        // TODO: switch by property factory address
        let property: Property = create_channel_state_object_for_variables(decider, inputs);
        QuantifierResult::new(vec![QuantifierResultItem::Property(property)], true)
    }
}
