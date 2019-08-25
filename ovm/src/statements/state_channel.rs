use crate::db::Message;
use crate::types::{
    AndDeciderInput, ForAllSuchThatInput, HasLowerNonceInput, Integer, Placeholder, Property,
    PropertyFactory, Quantifier, QuantifierResultItem, SignedByInput,
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
        Quantifier::SignedByQuantifier(Placeholder::new("my_address")),
        Placeholder::new("message"),
        Property::HasLowerNonceDecider(HasLowerNonceInput::new(
            Placeholder::new("message"),
            Placeholder::new("upper_nonce"),
        )),
    )));
    let right_property = Property::SignedByDecider(SignedByInput::new(
        Placeholder::new("last_message"),
        Placeholder::new("counter_party_address"),
    ));
    Property::AndDecider(Box::new(AndDeciderInput::new(
        left_property,
        right_property,
    )))
}
