use crate::types::{Property, PropertyInput};
use crate::DeciderManager;
use abi_utils::Integer;
use bytes::Bytes;
use ethereum_types::Address;
use plasma_core::data_structure::Range;

pub fn create_atomic_state(
    block_number: Integer,
    deposit_contract_address: Address,
    coin_range: Range,
    corresponding_inputs: Vec<PropertyInput>,
    property1: Property,
    property2: Property,
) -> Property {
    /*
     * There exists corresponding_property = create_channel(counter_party_address, my_address):
     *   There exists correspondent such that correspondent = create_state_update(corresponding_range, corresponding_property):
     *     Or(
     *       And(IncludedAt(correspondent), property1),
     *       And(Not(IncludedAt(correspondent)), property2)
     *     )
     *
     */
    DeciderManager::there_exists_such_that(vec![
        PropertyInput::ConstantProperty(DeciderManager::q_property(corresponding_inputs)),
        PropertyInput::ConstantBytes(Bytes::from("property")),
        PropertyInput::ConstantProperty(DeciderManager::there_exists_such_that(vec![
            PropertyInput::ConstantProperty(DeciderManager::q_state_update(vec![
                PropertyInput::ConstantInteger(block_number),
                PropertyInput::ConstantAddress(deposit_contract_address),
                PropertyInput::ConstantRange(coin_range),
                PropertyInput::Placeholder(Bytes::from("property")),
            ])),
            PropertyInput::ConstantBytes(Bytes::from("state_update")),
            PropertyInput::ConstantProperty(DeciderManager::or_decider(
                DeciderManager::and_decider(
                    DeciderManager::included_at_block_decider(vec![
                        PropertyInput::ConstantInteger(block_number),
                        PropertyInput::Placeholder(Bytes::from("state_update")),
                    ]),
                    property1,
                ),
                DeciderManager::and_decider(
                    DeciderManager::not_decider(DeciderManager::included_at_block_decider(vec![
                        PropertyInput::ConstantInteger(block_number),
                        PropertyInput::Placeholder(Bytes::from("state_update")),
                    ])),
                    property2,
                ),
            )),
        ])),
    ])
}
