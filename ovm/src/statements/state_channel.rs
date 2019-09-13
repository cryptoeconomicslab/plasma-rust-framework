use crate::db::Message;
use crate::types::{InputType, Integer, Property};
use crate::DeciderManager;
use bytes::Bytes;
use ethereum_types::Address;
use plasma_core::data_structure::abi::Encodable;

pub fn create_state_channel_property(
    my_address: Address,
    counter_party_address: Address,
    latest_message: Message,
) -> Property {
    let upper_nonce = Integer(latest_message.nonce.0 + 1);
    let left_property = DeciderManager::for_all_such_that_decider(
        DeciderManager::q_signed_by(vec![InputType::ConstantAddress(my_address)]),
        Bytes::from("message"),
        DeciderManager::has_lower_nonce_decider(vec![
            InputType::Placeholder(Bytes::from("message")),
            InputType::ConstantInteger(upper_nonce),
        ]),
    );
    let right_property = DeciderManager::signed_by_decider(vec![
        InputType::ConstantBytes(Bytes::from(latest_message.to_abi())),
        InputType::ConstantAddress(counter_party_address),
    ]);
    DeciderManager::and_decider(left_property, right_property)
}
