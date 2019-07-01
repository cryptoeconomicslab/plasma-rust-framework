use crate::traits::event_db::EventDb;

use ethabi::Hash;
use std::collections::HashMap;

#[derive(Default)]
pub struct MemoryDbImpl {
    last_logged_blocks: HashMap<Hash, u64>,
    seen_events: HashMap<Hash, bool>,
}

impl MemoryDbImpl {
    pub fn new() -> MemoryDbImpl {
        Default::default()
    }
}

impl EventDb<Hash> for MemoryDbImpl {
    fn get_last_logged_block(&self, topic_hash: Hash) -> Option<u64> {
        match self.last_logged_blocks.get(&topic_hash) {
            Some(block_number) => Some(*block_number),
            None => None,
        }
    }

    fn set_last_logged_block(&mut self, topic_hash: Hash, block_number: u64) {
        self.last_logged_blocks.insert(topic_hash, block_number);
    }

    fn get_event_seen(&self, event_hash: Hash) -> bool {
        match self.seen_events.get(&event_hash) {
            Some(seen) => *seen,
            None => false,
        }
    }

    fn set_event_seen(&mut self, event_hash: Hash) {
        self.seen_events.insert(event_hash, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_logged_block() {
        let mut db = MemoryDbImpl::new();
        let k = Hash::random();
        assert_eq!(db.get_last_logged_block(k), None);
        db.set_last_logged_block(k, 1);
        assert_eq!(db.get_last_logged_block(k), Some(1));
    }

    #[test]
    fn test_event_seen() {
        let mut db = MemoryDbImpl::new();
        let k = Hash::random();
        assert_eq!(db.get_event_seen(k), false);
        db.set_event_seen(k);
        assert_eq!(db.get_event_seen(k), true);
    }

}
