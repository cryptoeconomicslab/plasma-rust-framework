use crate::db::Message;
use crate::types::{
    AndDeciderInput, ForAllSuchThatInput, HasLowerNonceInput, InputType, Integer, Property,
    Quantifier, SignedByInput,
};
use bytes::Bytes;
use ethereum_types::Address;
use plasma_core::data_structure::abi::Encodable;

pub fn create_state_channel_property(
    my_address: Address,
    counter_party_address: Address,
    latest_message: Message,
) -> Property {
    let upper_nonce = Integer(latest_message.nonce.0 + 1);
    let left_property = Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
        Quantifier::SignedByQuantifier(InputType::ConstantAddress(my_address)),
        Bytes::from("message"),
        Property::HasLowerNonceDecider(HasLowerNonceInput::new(
            InputType::placeholder("message"),
            InputType::ConstantInteger(upper_nonce),
        )),
    )));
    let right_property = Property::SignedByDecider(SignedByInput::new(
        InputType::ConstantBytes(Bytes::from(latest_message.to_abi())),
        InputType::ConstantAddress(counter_party_address),
    ));
    Property::AndDecider(Box::new(AndDeciderInput::new(
        left_property,
        right_property,
    )))
}
