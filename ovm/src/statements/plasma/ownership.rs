use crate::property_executor::PropertyExecutor;
use crate::types::{Property, PropertyInput};
use crate::DeciderManager;
use bytes::Bytes;
use ethereum_types::Address;
use plasma_db::traits::kvs::KeyValueStore;

pub fn create_ownership_state_object_for_variables<KVS: KeyValueStore>(
    decider: &PropertyExecutor<KVS>,
    inputs: &[PropertyInput],
) -> Property {
    let to_address = decider.get_variable(&inputs[1]).to_address();
    create_ownership_state_object(to_address)
}

/// Claim for ownership
pub fn create_ownership_state_object(to_address: Address) -> Property {
    /*
     * There exists tx such that state_update.deprecate(tx):
     *   SignedBy(tx, to_address).
     */
    DeciderManager::there_exists_such_that(vec![
        PropertyInput::ConstantProperty(DeciderManager::q_tx(vec![PropertyInput::Placeholder(
            Bytes::from("state_update"),
        )])),
        PropertyInput::ConstantBytes(Bytes::from("tx")),
        PropertyInput::ConstantProperty(DeciderManager::signed_by_decider(vec![
            PropertyInput::ConstantAddress(to_address),
            PropertyInput::Placeholder(Bytes::from("tx")),
        ])),
    ])
}
