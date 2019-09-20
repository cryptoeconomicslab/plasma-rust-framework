use crate::property_executor::PropertyExecutor;
use crate::types::{
    Integer, Property, PropertyInput, QuantifierResult, QuantifierResultItem, StateUpdate,
};
use crate::DeciderManager;
use bytes::Bytes;
use ethereum_types::Address;
use plasma_core::data_structure::Range;
use plasma_db::traits::kvs::KeyValueStore;

pub struct SwapQuantifier {}

impl Default for SwapQuantifier {
    fn default() -> Self {
        Self {}
    }
}

impl SwapQuantifier {
    pub fn get_all_quantified<KVS>(
        decider: &PropertyExecutor<KVS>,
        inputs: &[PropertyInput],
    ) -> QuantifierResult
    where
        KVS: KeyValueStore,
    {
        let state_update = decider.get_variable(&inputs[0]).to_state_update();
        let previous_address = decider.get_variable(&inputs[1]).to_address();
        let next_address = decider.get_variable(&inputs[2]).to_address();
        let token_address = decider.get_variable(&inputs[3]).to_address();
        let corresponding_coin_range = decider.get_variable(&inputs[4]).to_range();
        let corresponding_block_number = decider.get_variable(&inputs[5]).to_integer();
        let corresponding_state_update = StateUpdate::new(
            corresponding_block_number,
            corresponding_coin_range,
            create_swap_order_property(
                previous_address,
                next_address,
                token_address,
                state_update.get_range(),
                state_update.get_block_number(),
            ),
        );
        QuantifierResult::new(
            vec![QuantifierResultItem::StateUpdate(
                corresponding_state_update,
            )],
            true,
        )
    }
}

pub fn create_swap_order_property(
    next_address: Address,
    previous_address: Address,
    token_address: Address,
    corresponding_coin_range: Range,
    corresponding_block_number: Integer,
) -> Property {
    /*
     * swap(state_update, previous_owner, next_owner, corresponding_token_address, corresponding_coin_range. corresponding_block_number)
     * := There exists tx such that state_update.is_same_range(tx):
     *      There exists corresponding such that swap(state_update, next_owner, previous_owner, corresponding_token_address, corresponding_coin_range. corresponding_block_number):
     *        Or(And(
     *          SignedBy(tx, next_owner).
     *          IsIncludedAt(corresponding)
     *        ),And(
     *          SignedBy(tx, previous_owner).
     *          Not(IsIncludedAt(corresponding))
     *        ))
     */
    DeciderManager::there_exists_such_that(vec![
        PropertyInput::ConstantProperty(DeciderManager::q_tx(vec![PropertyInput::Placeholder(
            Bytes::from("state_update"),
        )])),
        PropertyInput::ConstantBytes(Bytes::from("tx")),
        PropertyInput::ConstantProperty(DeciderManager::there_exists_such_that(vec![
            PropertyInput::ConstantProperty(DeciderManager::q_swap(vec![
                PropertyInput::Placeholder(Bytes::from("state_update")),
                PropertyInput::ConstantAddress(previous_address),
                PropertyInput::ConstantAddress(next_address),
                PropertyInput::ConstantAddress(token_address),
                PropertyInput::ConstantRange(corresponding_coin_range),
                PropertyInput::ConstantInteger(corresponding_block_number),
            ])),
            PropertyInput::ConstantBytes(Bytes::from("corresponding")),
            PropertyInput::ConstantProperty(DeciderManager::or_decider(
                DeciderManager::and_decider(
                    DeciderManager::signed_by_decider(vec![
                        PropertyInput::ConstantAddress(next_address),
                        PropertyInput::Placeholder(Bytes::from("tx")),
                    ]),
                    DeciderManager::included_at_block_decider(vec![PropertyInput::Placeholder(
                        Bytes::from("corresponding"),
                    )]),
                ),
                DeciderManager::and_decider(
                    DeciderManager::signed_by_decider(vec![
                        PropertyInput::ConstantAddress(previous_address),
                        PropertyInput::Placeholder(Bytes::from("tx")),
                    ]),
                    DeciderManager::not_decider(DeciderManager::included_at_block_decider(vec![
                        PropertyInput::Placeholder(Bytes::from("corresponding")),
                    ])),
                ),
            )),
        ])),
    ])
}
