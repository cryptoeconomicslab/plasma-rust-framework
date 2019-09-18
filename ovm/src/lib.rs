#[macro_use]
extern crate lazy_static;

pub mod db;
pub mod deciders;
pub mod error;
pub mod property_executor;
pub mod quantifiers;
pub mod statements;
pub mod types;
pub mod utils;

pub use self::property_executor::{DecideMixin, DeciderManager};

#[cfg(test)]
mod tests {

    use crate::db::{HashPreimageDb, Message, SignedByDb};
    use crate::deciders::preimage_exists_decider::Verifier;
    use crate::deciders::SignVerifier;
    use crate::property_executor::PropertyExecutor;
    use crate::statements::{create_plasma_property, create_state_channel_property};
    use crate::types::{Decision, Integer, PropertyInput};
    use crate::DeciderManager;
    use bytes::Bytes;
    use ethereum_types::Address;
    use ethsign::SecretKey;
    use plasma_core::data_structure::abi::Encodable;
    use plasma_core::data_structure::Range;
    use plasma_db::impls::kvs::CoreDbMemoryImpl;
    use plasma_db::traits::kvs::KeyValueStore;

    fn store_preimage<KVS: KeyValueStore>(decider: &PropertyExecutor<KVS>) {
        let db = HashPreimageDb::new(decider.get_db());
        for i in 0..10 {
            let integer = Integer(i);
            assert!(db
                .store_witness(Verifier::static_hash(&integer.into()), &integer.into())
                .is_ok());
        }
    }

    ///
    /// ```ignore
    /// ForAllSuchThat(nonce, IntegerRangeQuantifier(0, 10), PropertyFactory((nonce) => {
    ///   PreimageExistsDecider(nonce)
    /// }))
    /// ```
    ///
    #[test]
    fn test_decide_range_and_preimage() {
        let property = DeciderManager::for_all_such_that_decider(
            DeciderManager::q_range(vec![PropertyInput::ConstantRange(Range::new(0, 10))]),
            Bytes::from("n"),
            DeciderManager::for_all_such_that_decider(
                DeciderManager::q_hash(vec![PropertyInput::Placeholder(Bytes::from("n"))]),
                Bytes::from("h"),
                DeciderManager::preimage_exists_decider(vec![PropertyInput::Placeholder(
                    Bytes::from("h"),
                )]),
            ),
        );
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        store_preimage(&decider);
        let decided: Decision = decider.decide(&property).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }

    /// Test to fail
    #[test]
    #[should_panic]
    fn test_fail_to_decide_range_and_preimage() {
        let property = DeciderManager::for_all_such_that_decider(
            DeciderManager::q_range(vec![PropertyInput::ConstantRange(Range::new(0, 10))]),
            Bytes::from("n"),
            DeciderManager::for_all_such_that_decider(
                DeciderManager::q_hash(vec![PropertyInput::Placeholder(Bytes::from("n"))]),
                Bytes::from("h"),
                DeciderManager::preimage_exists_decider(vec![PropertyInput::Placeholder(
                    Bytes::from("h"),
                )]),
            ),
        );
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        let decided_result = decider.decide(&property);
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
        let property = DeciderManager::for_all_such_that_decider(
            DeciderManager::q_less_than(vec![PropertyInput::ConstantInteger(Integer(10))]),
            Bytes::from("n"),
            DeciderManager::for_all_such_that_decider(
                DeciderManager::q_hash(vec![PropertyInput::Placeholder(Bytes::from("n"))]),
                Bytes::from("h"),
                DeciderManager::preimage_exists_decider(vec![PropertyInput::Placeholder(
                    Bytes::from("h"),
                )]),
            ),
        );
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        store_preimage(&decider);
        let decided: Decision = decider.decide(&property).unwrap();
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
        let alice: Address = secret_key_alice.public().address().into();
        let bob: Address = secret_key_bob.public().address().into();
        let channel_id = Bytes::from("channel_id");
        let channel_message = Message::new(channel_id, Integer(10), Bytes::from("state_update"));
        let message = Bytes::from(channel_message.to_abi());
        let signature = SignVerifier::sign(&secret_key_bob, &message);
        let property = create_state_channel_property(alice, bob, channel_message.clone());
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        let db = SignedByDb::new(decider.get_db());
        assert!(db.store_witness(bob, message, signature).is_ok());
        let decided: Decision = decider.decide(&property).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }

    /// plasma
    #[test]
    fn test_fail_to_decide_plasma_checkpoint() {
        let block_number = Integer(10);
        let range = Range::new(0, 100);
        let checkpoint_property = create_plasma_property(block_number, range);
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        let result = decider.decide(&checkpoint_property);
        // faid to decide because no local decision
        assert!(result.is_err());
    }
}
