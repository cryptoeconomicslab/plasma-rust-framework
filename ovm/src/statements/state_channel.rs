use crate::db::Message;
use crate::types::{Integer, Property, PropertyInput};
use crate::DeciderManager;
use abi_utils::Encodable;
use bytes::Bytes;
use ethereum_types::Address;

pub fn create_state_channel_property(
    my_address: Address,
    counter_party_address: Address,
    latest_message: Message,
) -> Property {
    let upper_nonce = Integer(latest_message.nonce.0 + 1);
    let left_property = DeciderManager::for_all_such_that_decider(
        DeciderManager::q_signed_by(vec![PropertyInput::ConstantAddress(my_address)]),
        Bytes::from("message"),
        DeciderManager::has_lower_nonce_decider(vec![
            PropertyInput::Placeholder(Bytes::from("message")),
            PropertyInput::ConstantInteger(upper_nonce),
        ]),
    );
    let right_property = DeciderManager::signed_by_decider(vec![
        PropertyInput::ConstantAddress(counter_party_address),
        PropertyInput::ConstantBytes(Bytes::from(latest_message.to_abi())),
    ]);
    DeciderManager::and_decider(left_property, right_property)
}
