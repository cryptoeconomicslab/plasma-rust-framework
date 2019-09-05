use crate::db::Message;
use crate::types::{
    AndDeciderInput, ForAllSuchThatInput, HasLowerNonceInput, Integer, Property, PropertyFactory,
    Quantifier, QuantifierResultItem, SignedByInput,
};
use bytes::Bytes;
use ethereum_types::Address;
use plasma_core::data_structure::abi::{Decodable, Encodable};

pub fn create_state_channel_property(
    my_address: Address,
    counter_party_address: Address,
    latest_message: Message,
) -> Property {
    let upper_nonce = Integer(latest_message.nonce.0 + 1);
    let left_property = Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
        Quantifier::SignedByQuantifier(my_address),
        Some(PropertyFactory::new(Box::new(move |item| {
            if let QuantifierResultItem::Bytes(message) = item {
                Property::HasLowerNonceDecider(HasLowerNonceInput::new(
                    Message::from_abi(&message).unwrap(),
                    upper_nonce,
                ))
            } else {
                panic!("invalid type in PropertyFactory");
            }
        }))),
    )));
    let right_property = Property::SignedByDecider(SignedByInput::new(
        Bytes::from(latest_message.to_abi()),
        counter_party_address,
    ));
    Property::AndDecider(Box::new(AndDeciderInput::new(
        left_property,
        right_property,
    )))
}
