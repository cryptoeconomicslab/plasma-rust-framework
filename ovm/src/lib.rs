#[macro_use]
extern crate lazy_static;

pub mod db;
pub mod deciders;
pub mod error;
pub mod property_executor;
pub mod quantifiers;
pub mod statements;
pub mod types;

pub use self::property_executor::DecideMixin;

#[cfg(test)]
mod tests {

    use crate::deciders::preimage_exists_decider::Verifier;
    use crate::deciders::SignVerifier;
    use crate::property_executor::PropertyExecutor;
    use crate::statements::create_plasma_property;
    use crate::types::{
        AndDeciderInput, Decision, ForAllSuchThatInput, HasLowerNonceInput, Integer,
        IntegerRangeQuantifierInput, PreimageExistsInput, Property, PropertyFactory, Quantifier,
        QuantifierResultItem, SignedByInput, Witness, WitnessFactory,
    };
    use bytes::Bytes;
    use ethereum_types::Address;
    use ethsign::SecretKey;
    use plasma_core::data_structure::Range;
    use plasma_db::impls::kvs::CoreDbLevelDbImpl;

    ///
    /// ```ignore
    /// ForAllSuchThat(nonce, IntegerRangeQuantifier(0, 10), PropertyFactory((nonce) => {
    ///   PreimageExistsDecider(nonce)
    /// }))
    /// ```
    ///
    #[test]
    fn test_decide_range_and_preimage() {
        let property = Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
            Quantifier::IntegerRangeQuantifier(IntegerRangeQuantifierInput::new(0, 10)),
            Some(PropertyFactory::new(Box::new(|item| {
                if let QuantifierResultItem::Integer(number) = item {
                    Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(
                        Verifier::static_hash(&number.into()),
                    )))
                } else {
                    panic!("invalid type in PropertyFactory");
                }
            }))),
            Some(WitnessFactory::new(Box::new(|item| {
                if let QuantifierResultItem::Integer(number) = item {
                    Witness::Bytes(number.into())
                } else {
                    panic!("invalid type in PropertyFactory");
                }
            }))),
        )));
        let decider: PropertyExecutor<CoreDbLevelDbImpl> = Default::default();
        let decided: Decision = decider.decide(&property, None).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }

    /// Test to fail
    #[test]
    fn test_fail_to_decide_range_and_preimage() {
        let property = Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
            Quantifier::IntegerRangeQuantifier(IntegerRangeQuantifierInput::new(0, 10)),
            Some(PropertyFactory::new(Box::new(|item| {
                if let QuantifierResultItem::Integer(number) = item {
                    Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(
                        Verifier::static_hash(&number.into()),
                    )))
                } else {
                    panic!("invalid type in PropertyFactory");
                }
            }))),
            Some(WitnessFactory::new(Box::new(|_item| {
                Witness::Bytes(Bytes::from(&b"aaa"[..]))
            }))),
        )));
        let decider: PropertyExecutor<CoreDbLevelDbImpl> = Default::default();
        let decided_result = decider.decide(&property, None);
        assert_eq!(decided_result.is_ok(), false);
    }

    ///
    /// ```ignore
    /// ForAllSuchThat(nonce, LessThanQuantifier(10), PropertyFactory((nonce) => {
    ///   PreimageExistsDecider(nonce)
    /// }))
    /// ```
    ///
    #[test]
    fn test_decide_less_than_and_preimage() {
        let property = Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
            Quantifier::NonnegativeIntegerLessThanQuantifier(Integer(10)),
            Some(PropertyFactory::new(Box::new(|item| {
                if let QuantifierResultItem::Integer(number) = item {
                    Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(
                        Verifier::static_hash(&number.into()),
                    )))
                } else {
                    panic!("invalid type in PropertyFactory");
                }
            }))),
            Some(WitnessFactory::new(Box::new(|item| {
                if let QuantifierResultItem::Integer(number) = item {
                    Witness::Bytes(number.into())
                } else {
                    panic!("invalid type in PropertyFactory");
                }
            }))),
        )));
        let decider: PropertyExecutor<CoreDbLevelDbImpl> = Default::default();
        let decided: Decision = decider.decide(&property, None).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }

    /// state channel
    #[test]
    fn test_state_channel() {
        let raw_key_alice =
            hex::decode("c87509a1c067bbde78beb793e6fa76530b6382a4c0241e5e4a9ec0a0f44dc0d3")
                .unwrap();
        let raw_key_bob =
            hex::decode("ae6ae8e5ccbfb04590405997ee2d52d2b330726137b875053c36d94e974d162f")
                .unwrap();
        let secret_key_alice = SecretKey::from_raw(&raw_key_alice).unwrap();
        let secret_key_bob = SecretKey::from_raw(&raw_key_bob).unwrap();
        let message = Bytes::from("state_update");
        let signature = SignVerifier::sign(&secret_key_bob, &message);
        let alice: Address = secret_key_alice.public().address().into();
        let bob: Address = secret_key_bob.public().address().into();
        let _nonce = Integer(10);
        let left_property = Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
            Quantifier::SignedByQuantifier(alice),
            Some(PropertyFactory::new(Box::new(|item| {
                if let QuantifierResultItem::Message(message) = item {
                    Property::HasLowerNonceDecider(HasLowerNonceInput::new(message, Integer(11)))
                } else {
                    panic!("invalid type in PropertyFactory");
                }
            }))),
            None,
        )));
        let right_property =
            Property::SignedByDecider(SignedByInput::new(Bytes::from("state_update"), bob));
        let property = Property::AndDecider(Box::new(AndDeciderInput::new(
            left_property,
            Witness::Bytes("".into()),
            right_property,
            Witness::Bytes(signature),
        )));

        let decider: PropertyExecutor<CoreDbLevelDbImpl> = Default::default();
        let decided: Decision = decider.decide(&property, None).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }

    /// plasma
    #[test]
    fn test_fail_to_decide_plasma_checkpoint() {
        let block_number = Integer(10);
        let range = Range::new(0, 100);
        let checkpoint_property = create_plasma_property(block_number, range);
        let decider: PropertyExecutor<CoreDbLevelDbImpl> = Default::default();
        let result = decider.decide(&checkpoint_property, None);
        // faid to decide because no local decision
        assert!(result.is_err());
    }
}
