use crate::error::Error;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseDbKey(Vec<u8>);

impl BaseDbKey {
    pub fn new(key: Vec<u8>) -> Self {
        BaseDbKey(key)
    }

    pub fn get(&self) -> &[u8] {
        &self.0
    }

    pub fn concat(&self, key: &BaseDbKey) -> BaseDbKey {
        BaseDbKey([self.0.as_slice(), key.0.as_slice()].concat())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Batch {
    BatchPut { key: BaseDbKey, value: Vec<u8> },
    BatchDel { key: BaseDbKey },
}

impl Batch {
    pub fn new_put(key: BaseDbKey, value: &[u8]) -> Self {
        Batch::BatchPut {
            key,
            value: value.to_vec(),
        }
    }
    pub fn new_del(key: BaseDbKey) -> Self {
        Batch::BatchDel { key }
    }
}

pub struct Bucket<'a, T: Iterator<Item = Vec<u8>>> {
    prefix: BaseDbKey,
    store: &'a KeyValueStore<T>,
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

pub trait KeyValueStore<T: Iterator<Item = Vec<u8>>> {
    fn get(&self, key: &BaseDbKey) -> Result<Option<Box<[u8]>>, Error>;
    fn put(&self, key: &BaseDbKey, value: &[u8]) -> Result<(), Error>;
    fn del(&self, key: &BaseDbKey) -> Result<(), Error>;
    fn has(&self, key: &BaseDbKey) -> Result<bool, Error>;
    fn batch(&self, operations: &[Batch]) -> Result<(), Error>;
    fn iterator(&self, prefix: &BaseDbKey) -> Result<T, Error>;
    fn bucket(&self, prefix: &BaseDbKey) -> Box<Bucket<T>>;
}

impl<'a, T> Bucket<'a, T>
where
    T: Iterator<Item = Vec<u8>>,
{
    pub fn new(prefix: BaseDbKey, store: &'a KeyValueStore<T>) -> Self {
        Self { prefix, store }
    }
}

impl<'a, T> KeyValueStore<T> for Bucket<'a, T>
where
    T: Iterator<Item = Vec<u8>>,
{
    fn get(&self, key: &BaseDbKey) -> Result<Option<Box<[u8]>>, Error> {
        self.store.get(&self.prefix.concat(key))
    }
    fn put(&self, key: &BaseDbKey, value: &[u8]) -> Result<(), Error> {
        self.store.put(&self.prefix.concat(key), value)
    }
    fn del(&self, key: &BaseDbKey) -> Result<(), Error> {
        self.store.del(&self.prefix.concat(key))
    }
    fn has(&self, key: &BaseDbKey) -> Result<bool, Error> {
        self.store.has(&self.prefix.concat(key))
    }
    fn batch(&self, operations: &[Batch]) -> Result<(), Error> {
        let new_ops: Vec<Batch> = operations
            .iter()
            .map(|op| match op {
                Batch::BatchPut { key, value } => Batch::new_put(self.prefix.concat(key), value),
                Batch::BatchDel { key } => Batch::new_del(self.prefix.concat(key)),
            })
            .collect();
        self.store.batch(&new_ops)
    }
    fn iterator(&self, prefix: &BaseDbKey) -> Result<T, Error> {
        self.store.iterator(&self.prefix.concat(prefix))
    }
    fn bucket(&self, prefix: &BaseDbKey) -> Box<Bucket<T>> {
        self.store.bucket(&self.prefix.concat(prefix))
    }
}
