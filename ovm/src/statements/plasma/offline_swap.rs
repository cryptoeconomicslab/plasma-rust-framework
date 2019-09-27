use crate::statements::plasma::create_plasma_checkpoint_property_for_variables;
use crate::types::{Integer, Property, PropertyInput};
use crate::DeciderManager;
use bytes::Bytes;
use ethereum_types::Address;
use plasma_core::data_structure::Range;

/// Pre-swap property for Plasma
pub fn create_pre_swap_state_object(
    my_address: Address,
    token_type: Address,
    amount: Integer,
) -> Property {
    /*
     * There exists tx such that state_update.is_same_coin_range(tx):
     *   Or(
     *     verify_state_transition(OFFLINE_SWAP, tx, token_type, amount),
     *     SignedBy(tx, my_address)
     *   )
     *
     */
    DeciderManager::there_exists_such_that(vec![
        PropertyInput::ConstantProperty(DeciderManager::q_tx(vec![PropertyInput::Placeholder(
            Bytes::from("state_update"),
        )])),
        PropertyInput::ConstantBytes(Bytes::from("tx")),
        PropertyInput::ConstantProperty(DeciderManager::or_decider(
            DeciderManager::verify_tx(vec![
                PropertyInput::Placeholder(Bytes::from("tx")),
                PropertyInput::ConstantAddress(token_type),
                PropertyInput::ConstantInteger(amount),
            ]),
            DeciderManager::signed_by_decider(vec![
                PropertyInput::ConstantAddress(my_address),
                PropertyInput::Placeholder(Bytes::from("tx")),
            ]),
        )),
    ])
}

pub fn create_offline_atomic_state(
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
     *       And(Checkpoint(b, c_token, c_range), IncludedAt(correspondent), property1),
     *       And(Not(And(Checkpoint(b, c_token, c_range), IncludedAt(correspondent))), property2)
     *     )
     *
     */
    DeciderManager::there_exists_such_that(vec![
        PropertyInput::ConstantProperty(DeciderManager::q_property(corresponding_inputs)),
        PropertyInput::ConstantBytes(Bytes::from("property")),
        PropertyInput::ConstantProperty(DeciderManager::there_exists_such_that(vec![
            PropertyInput::ConstantProperty(DeciderManager::q_state_update(vec![
                PropertyInput::Placeholder(Bytes::from("block")),
                PropertyInput::ConstantAddress(deposit_contract_address),
                PropertyInput::ConstantRange(coin_range),
                PropertyInput::Placeholder(Bytes::from("property")),
            ])),
            PropertyInput::ConstantBytes(Bytes::from("state_update")),
            PropertyInput::ConstantProperty(DeciderManager::or_decider(
                DeciderManager::and_decider(
                    DeciderManager::and_decider(
                        create_plasma_checkpoint_property_for_variables(
                            PropertyInput::Placeholder(Bytes::from("block")),
                            PropertyInput::ConstantAddress(deposit_contract_address),
                            PropertyInput::ConstantRange(coin_range),
                        ),
                        DeciderManager::included_at_block_decider(vec![
                            PropertyInput::Placeholder(Bytes::from("block")),
                            PropertyInput::Placeholder(Bytes::from("state_update")),
                        ]),
                    ),
                    property1,
                ),
                DeciderManager::and_decider(
                    DeciderManager::not_decider(DeciderManager::and_decider(
                        create_plasma_checkpoint_property_for_variables(
                            PropertyInput::Placeholder(Bytes::from("block")),
                            PropertyInput::ConstantAddress(deposit_contract_address),
                            PropertyInput::ConstantRange(coin_range),
                        ),
                        DeciderManager::included_at_block_decider(vec![
                            PropertyInput::Placeholder(Bytes::from("block")),
                            PropertyInput::Placeholder(Bytes::from("state_update")),
                        ]),
                    )),
                    property2,
                ),
            )),
        ])),
    ])
}

/// Swap property for Plasma
pub fn create_swap_state_object(
    my_address: Address,
    counter_party_address: Address,
    c_token_address: Address,
    c_range: Range,
) -> Property {
    /*
     * There exists tx such that state_update.is_same_coin_range(tx):
     *   Or(
     *     And(
     *       Checkpoint(b, c_token, c_range),
     *       IncludedAt(correspondent)
     *       SignedBy(tx, to_address)
     *     ),
     *     And(
     *       Not(
     *         And(
     *           Checkpoint(b, c_token, c_range)
     *         )
     *       ),
     *       SignedBy(tx, my_address))
     *   )
     *
     * For all b such that b < block_number:
     *   For all state_update such that state_update <- BlockRange(b, range):
     *     IsDeprecated(state_update)
     *
     * There exists corresponding_property = create_swap(counter_party_address, my_address):
     *   There exists correspondent such that correspondent = create_state_update(corresponding_range, corresponding_property):
     *     Or(
     *       And(IncludedAt(correspondent), property1),
     *       And(Not(IncludedAt(correspondent)), property2)
     *     )
     *
     */
    DeciderManager::there_exists_such_that(vec![
        PropertyInput::ConstantProperty(DeciderManager::q_tx(vec![PropertyInput::Placeholder(
            Bytes::from("state_update"),
        )])),
        PropertyInput::ConstantBytes(Bytes::from("tx")),
        PropertyInput::ConstantProperty(create_offline_atomic_state(
            c_token_address,
            c_range,
            vec![
                // TODO: This should be PropertyFactory address
                PropertyInput::ConstantInteger(Integer(2)),
                PropertyInput::ConstantAddress(counter_party_address),
                PropertyInput::ConstantAddress(my_address),
                PropertyInput::Placeholder(Bytes::from("state_update")),
            ],
            DeciderManager::signed_by_decider(vec![
                PropertyInput::ConstantAddress(counter_party_address),
                PropertyInput::Placeholder(Bytes::from("tx")),
            ]),
            DeciderManager::signed_by_decider(vec![
                PropertyInput::ConstantAddress(my_address),
                PropertyInput::Placeholder(Bytes::from("tx")),
            ]),
        )),
    ])
}
