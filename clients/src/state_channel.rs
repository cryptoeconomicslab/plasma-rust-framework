use bytes::Bytes;
use ethereum_types::Address;
use ovm::db::{ChannelDb, Message, MessageDb, SignedByDb};
use ovm::deciders::SignVerifier;
use ovm::property_executor::PropertyExecutor;
use ovm::statements::create_state_channel_property;
use ovm::types::{Decision, ImplicationProofElement, Property, SignedByInput, Witness};
use plasma_core::data_structure::abi::Encodable;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;

pub struct StateChannel<KVS: KeyValueStore + DatabaseTrait> {
    db: KVS,
}

impl<KVS: KeyValueStore + DatabaseTrait> StateChannel<KVS> {
    /// Called handling new message through pubsub network
    pub fn handle_message(&self, channel_message: Message, signature: Bytes) {
        let message = Bytes::from(channel_message.to_abi());
        let counter_party = SignVerifier::recover(&signature, &message);
        let witness = Witness::Bytes(signature);
        let sign_input = SignedByInput::new(Bytes::from(channel_message.to_abi()), counter_party);
        let db = SignedByDb::new(&self.db);
        assert!(db.store_witness(&sign_input, &witness).is_ok());
    }

    /// Gets exit claim
    pub fn exit_claim(
        &self,
        channel_id: &Bytes,
        my_address: Address,
        counter_party: Address,
    ) -> Vec<u8> {
        self.get_exit_claim(channel_id, my_address, counter_party)
            .to_abi()
    }

    /// Called when channel is opened
    pub fn handle_opening_channel(&self, channel_message: &Message) {
        let message_db: MessageDb<KVS> = (&self.db).into();
        assert!(message_db.store_message(channel_message).is_ok());
    }

    /// Called when someone exit state
    pub fn handle_exit(
        &self,
        channel_id: &Bytes,
        claim: &Property,
    ) -> Vec<ImplicationProofElement> {
        let decider: PropertyExecutor<KVS> = Default::default();
        let decision: Decision = decider.decide(&claim).unwrap();
        if decision.get_outcome() {
            let channel_db: ChannelDb<KVS> = (&self.db).into();
            assert!(channel_db.mark_exited(channel_id).is_ok());
        }
        decision.get_implication_proof().clone()
    }

    pub fn check_claim(
        &self,
        channel_id: &Bytes,
        my_address: Address,
        counter_party: Address,
    ) -> Option<Decision> {
        let property = self.get_exit_claim(channel_id, my_address, counter_party);
        let decider: PropertyExecutor<KVS> = Default::default();
        decider.decide(&property).ok()
    }

    fn get_exit_claim(
        &self,
        channel_id: &Bytes,
        my_address: Address,
        counter_party: Address,
    ) -> Property {
        let message_db: MessageDb<KVS> = (&self.db).into();
        let most_recent_message = message_db.get_most_recent_message(channel_id);
        if let Some(most_recent_message) = most_recent_message {
            create_state_channel_property(my_address, counter_party, most_recent_message)
        } else {
            panic!("There are no messages!!")
        }
    }
}
