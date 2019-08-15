use super::message::Message;
use plasma_db::traits::kvs::KeyValueStore;

pub struct MessageDb<KVS> {
    db: KVS,
}

impl<KVS> MessageDb<KVS>
where
    KVS: KeyValueStore,
{
    pub fn new(db: KVS) -> Self {
        Self { db }
    }

    // TODO: change message
    pub fn persist_message(&self, msg: Message) {
        println!("Message persisted: {:?}", msg);
        // TODO: persist message
    }
}
