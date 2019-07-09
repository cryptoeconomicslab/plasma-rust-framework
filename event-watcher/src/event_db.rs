use ethabi::Hash;
use plasma_db::traits::kvs::{BaseDbKey, KeyValueStore};

pub trait EventDb {
    fn get_last_logged_block(&self, topic_hash: Hash) -> Option<u64>;
    fn set_last_logged_block(&mut self, topic_hash: Hash, block_number: u64);
    fn get_event_seen(&self, event_hash: Hash) -> bool;
    fn set_event_seen(&mut self, event_hash: Hash);
}

pub struct EventDbImpl<KVS> {
    db: KVS,
}

impl<KVS> From<KVS> for EventDbImpl<KVS>
where
    KVS: KeyValueStore,
{
    fn from(db: KVS) -> Self {
        Self { db }
    }
}

impl<KVS> EventDb for EventDbImpl<KVS>
where
    KVS: KeyValueStore,
{
    fn get_last_logged_block(&self, topic_hash: Hash) -> Option<u64> {
        match self.db.get(&BaseDbKey::new(topic_hash.0.to_vec())) {
            Ok(Some(v)) => Some(rlp::decode(&v[..]).unwrap()),
            Ok(None) => None,
            Err(_) => None,
        }
    }

    fn set_last_logged_block(&mut self, topic_hash: Hash, block_number: u64) {
        let _ = self.db.put(
            &BaseDbKey::new(topic_hash.0.to_vec()),
            &rlp::encode(&block_number),
        );
    }

    fn get_event_seen(&self, event_hash: Hash) -> bool {
        match self.db.get(&BaseDbKey::new(event_hash.0.to_vec())) {
            Ok(Some(v)) => rlp::decode(&v[..]).unwrap(),
            Ok(None) => false,
            Err(_) => false,
        }
    }

    fn set_event_seen(&mut self, event_hash: Hash) {
        let _ = self
            .db
            .put(&BaseDbKey::new(event_hash.0.to_vec()), &rlp::encode(&true));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plasma_db::impls::kvs::memory::CoreDbMemoryImpl;
    use plasma_db::traits::DatabaseTrait;

    #[test]
    fn test_last_logged_block() {
        let kvs = CoreDbMemoryImpl::open("kvs");
        let mut db = EventDbImpl::from(kvs);
        let k = Hash::random();
        assert_eq!(db.get_last_logged_block(k), None);
        db.set_last_logged_block(k, 1);
        assert_eq!(db.get_last_logged_block(k), Some(1));
    }

    #[test]
    fn test_event_seen() {
        let kvs = CoreDbMemoryImpl::open("kvs");
        let mut db = EventDbImpl::from(kvs);
        let k = Hash::random();
        assert_eq!(db.get_event_seen(k), false);
        db.set_event_seen(k);
        assert_eq!(db.get_event_seen(k), true);
    }

}
