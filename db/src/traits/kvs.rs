use crate::error::Error;

pub enum Batch<'a> {
    BatchPut { key: &'a [u8], value: &'a [u8] },
    BatchDel { key: &'a [u8] },
}

pub struct KeyValue {
    key: Box<[u8]>,
    value: Box<[u8]>,
}

impl KeyValue {
    pub fn get_key(&self) -> &[u8] {
        &self.key
    }
    pub fn get_value(&self) -> &[u8] {
        &self.value
    }
}

pub trait KvsIterator {
    fn next(&self) -> Result<KeyValue, Error>;
}

pub trait KeyValueStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error>;
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error>;
    fn del(&self, key: &[u8]) -> Result<(), Error>;
    fn has(&self, key: &[u8]) -> Result<bool, Error>;
    fn batch(&self, operations: &[Batch]) -> Result<(), Error>;
    fn iterator(&self, prefix: &[u8]) -> Result<Box<KvsIterator>, Error>;
}
