pub trait EventDb<T> {
    fn get_last_logged_block(&self, topic_hash: T) -> Option<u64>;
    fn set_last_logged_block(&mut self, topic_hash: T, block_number: u64);
    fn get_event_seen(&self, event_hash: T) -> bool;
    fn set_event_seen(&mut self, event_hash: T);
}
