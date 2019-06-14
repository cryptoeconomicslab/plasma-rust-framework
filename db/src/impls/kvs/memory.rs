use crate::error::Error;
use crate::traits::db::DatabaseTrait;
use crate::traits::kvs::{BaseDbKey, Batch, Bucket, KeyValue, KeyValueStore};
use parking_lot::RwLock;
use std::collections::BTreeMap;

pub struct CoreDbMemoryImpl {
    db: RwLock<BTreeMap<BaseDbKey, Vec<u8>>>,
}

impl DatabaseTrait for CoreDbMemoryImpl {
    fn open(_dbname: &str) -> Self {
        Self {
            db: RwLock::new(BTreeMap::new()),
        }
    }
    fn close(&self) {}
}

impl<'a, B> KeyValueStore<B> for CoreDbMemoryImpl {
    fn get(&self, key: &BaseDbKey) -> Result<Option<Vec<u8>>, Error> {
        Ok(self.db.read().get(key).map(|v| v.to_vec()))
    }
    fn put(&self, key: &BaseDbKey, value: &[u8]) -> Result<(), Error> {
        self.db.write().insert(key.clone(), value.to_vec());
        Ok(())
    }
    fn del(&self, key: &BaseDbKey) -> Result<(), Error> {
        self.db.write().remove(key);
        Ok(())
    }
    fn has(&self, _key: &BaseDbKey) -> Result<bool, Error> {
        Ok(true)
    }
    fn batch(&self, operations: &[Batch]) -> Result<(), Error> {
        let mut write_lock = self.db.write();
        for op in operations.iter() {
            match op {
                Batch::BatchPut { key, value } => write_lock.insert(key.clone(), value.clone()),
                Batch::BatchDel { key } => write_lock.remove(key),
            };
        }
        Ok(())
    }
    fn iter_all(
        &self,
        prefix: &BaseDbKey,
        mut f: Box<FnMut(&BaseDbKey, &Vec<u8>) -> bool>,
    ) -> Vec<KeyValue> {
        let read_lock = self.db.read();
        let iter = read_lock.iter();
        let mut result = vec![];
        for (k, v) in iter {
            if k > prefix {
                if f(&k, &v) {
                    result.push(KeyValue::new(k.clone(), v.clone()));
                    continue;
                } else {
                    break;
                }
            }
        }
        result
    }
    fn iter_all_map(
        &self,
        prefix: &BaseDbKey,
        mut f: Box<FnMut(&BaseDbKey, &Vec<u8>) -> Option<B>>,
    ) -> Vec<B> {
        let read_lock = self.db.read();
        let iter = read_lock.iter();
        let mut result = vec![];
        for (k, v) in iter {
            if k > prefix {
                if let Some(b) = f(&k, &v) {
                    result.push(b);
                    continue;
                } else {
                    break;
                }
            }
        }
        result
    }
    fn bucket(&self, prefix: &BaseDbKey) -> Bucket<B> {
        Bucket::new(prefix.clone(), self)
    }
}
