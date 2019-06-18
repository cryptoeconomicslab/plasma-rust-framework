use crate::error::Error;
use bytes::{BufMut, BytesMut};
use std::cmp::Ordering;

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

impl From<&[u8]> for BaseDbKey {
    fn from(array: &[u8]) -> Self {
        BaseDbKey::new(array.to_vec())
    }
}

impl From<u64> for BaseDbKey {
    fn from(n: u64) -> Self {
        let mut buf = BytesMut::with_capacity(64);
        buf.put_u64_le(n);
        BaseDbKey::new(buf.to_vec())
    }
}

impl PartialOrd for BaseDbKey {
    fn partial_cmp(&self, other: &BaseDbKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BaseDbKey {
    fn cmp(&self, other: &BaseDbKey) -> Ordering {
        self.0.cmp(&other.0)
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

pub struct Bucket<'a, B> {
    prefix: BaseDbKey,
    store: &'a KeyValueStore<B>,
}

pub struct KeyValue {
    key: BaseDbKey,
    value: Vec<u8>,
}

impl KeyValue {
    pub fn new(key: BaseDbKey, value: Vec<u8>) -> Self {
        KeyValue { key, value }
    }

    pub fn get_key(&self) -> &BaseDbKey {
        &self.key
    }
    pub fn get_value(&self) -> &Vec<u8> {
        &self.value
    }
}

pub trait KeyValueStore<B> {
    fn get(&self, key: &BaseDbKey) -> Result<Option<Vec<u8>>, Error>;
    fn put(&self, key: &BaseDbKey, value: &[u8]) -> Result<(), Error>;
    fn del(&self, key: &BaseDbKey) -> Result<(), Error>;
    fn has(&self, key: &BaseDbKey) -> Result<bool, Error>;
    fn batch(&self, operations: &[Batch]) -> Result<(), Error>;
    /// This is substitute of iter
    fn iter_all(
        &self,
        prefix: &BaseDbKey,
        f: Box<FnMut(&BaseDbKey, &Vec<u8>) -> bool>,
    ) -> Vec<KeyValue>;
    fn iter_all_map(
        &self,
        prefix: &BaseDbKey,
        f: Box<FnMut(&BaseDbKey, &Vec<u8>) -> Option<B>>,
    ) -> Vec<B>;
    fn bucket(&self, prefix: &BaseDbKey) -> Bucket<B>;
    fn root(&self) -> Bucket<B> {
        self.bucket(&b""[..].into())
    }
}

impl<'a, B> Bucket<'a, B> {
    pub fn new(prefix: BaseDbKey, store: &'a KeyValueStore<B>) -> Self {
        Self { prefix, store }
    }
}

impl<'a, B> KeyValueStore<B> for Bucket<'a, B> {
    fn get(&self, key: &BaseDbKey) -> Result<Option<Vec<u8>>, Error> {
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
    fn iter_all(
        &self,
        prefix: &BaseDbKey,
        f: Box<FnMut(&BaseDbKey, &Vec<u8>) -> bool>,
    ) -> Vec<KeyValue> {
        self.store.iter_all(&self.prefix.concat(prefix), f)
    }
    fn iter_all_map(
        &self,
        prefix: &BaseDbKey,
        f: Box<FnMut(&BaseDbKey, &Vec<u8>) -> Option<B>>,
    ) -> Vec<B> {
        self.store.iter_all_map(&self.prefix.concat(prefix), f)
    }
    fn bucket(&self, prefix: &BaseDbKey) -> Bucket<B> {
        self.store.bucket(&self.prefix.concat(prefix))
    }
}
