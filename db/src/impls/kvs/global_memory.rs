use crate::error::Error;
use crate::traits::db::DatabaseTrait;
use crate::traits::kvs::{BaseDbKey, Batch, Bucket, KeyValue, KeyValueStore};
use parking_lot::RwLock;
use std::collections::BTreeMap;

lazy_static! {
    static ref GLOBAL_DB: RwLock<BTreeMap<BaseDbKey, Vec<u8>>> = RwLock::new(BTreeMap::new());
}

pub struct GlobalMemoryDb {
    dbname: BaseDbKey,
}

impl GlobalMemoryDb {
    fn get_key(&self, key: &BaseDbKey) -> BaseDbKey {
        self.dbname.concat(key)
    }
}

impl DatabaseTrait for GlobalMemoryDb {
    fn open(dbname: &str) -> Self {
        Self {
            dbname: BaseDbKey::from(dbname),
        }
    }
    fn close(&self) {}
}

impl KeyValueStore for GlobalMemoryDb {
    fn get(&self, key: &BaseDbKey) -> Result<Option<Vec<u8>>, Error> {
        Ok(GLOBAL_DB.read().get(&self.get_key(key)).map(|v| v.to_vec()))
    }
    fn put(&self, key: &BaseDbKey, value: &[u8]) -> Result<(), Error> {
        GLOBAL_DB.write().insert(self.get_key(key), value.to_vec());
        Ok(())
    }
    fn del(&self, key: &BaseDbKey) -> Result<(), Error> {
        GLOBAL_DB.write().remove(&self.get_key(key));
        Ok(())
    }
    fn has(&self, _key: &BaseDbKey) -> Result<bool, Error> {
        Ok(true)
    }
    fn batch(&self, operations: &[Batch]) -> Result<(), Error> {
        let mut write_lock = GLOBAL_DB.write();
        for op in operations.iter() {
            match op {
                Batch::BatchPut { key, value } => {
                    write_lock.insert(self.get_key(key), value.clone())
                }
                Batch::BatchDel { key } => write_lock.remove(&self.get_key(key)),
            };
        }
        Ok(())
    }
    fn iter_all_with_prefix(
        &self,
        prefix: &BaseDbKey,
        start: &BaseDbKey,
        mut f: Box<dyn FnMut(&BaseDbKey, &Vec<u8>) -> bool>,
    ) -> Vec<KeyValue> {
        let pprefix = self.get_key(prefix);
        let read_lock = GLOBAL_DB.read();
        let iter = read_lock.iter();
        let mut result = vec![];
        for (k, v) in iter {
            if *k > pprefix.concat(start) {
                if k.0.starts_with(&pprefix.0) && f(&k, &v) {
                    result.push(KeyValue::new(k.clone(), v.clone()));
                    continue;
                } else {
                    break;
                }
            }
        }
        result
    }
    fn iter_all(
        &self,
        start: &BaseDbKey,
        mut f: Box<dyn FnMut(&BaseDbKey, &Vec<u8>) -> bool>,
    ) -> Vec<KeyValue> {
        let prefix = &self.get_key(start);
        let read_lock = GLOBAL_DB.read();
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
    fn bucket<'a>(&'a self, prefix: &BaseDbKey) -> Bucket<'a> {
        Bucket::new(prefix.clone(), self)
    }
}

#[cfg(test)]
mod tests {
    use super::GlobalMemoryDb;
    use crate::traits::db::DatabaseTrait;
    use crate::traits::kvs::{Bucket, KeyValueStore};

    #[test]
    fn test_bucket() {
        let core_db = GlobalMemoryDb::open("test");
        let root: Bucket = core_db.root();
        let bucket: Bucket = root.bucket(&b"a"[..].into());
        assert_eq!(bucket.put(&b"b"[..].into(), &b"value"[..]).is_ok(), true);
        let result = root.get(&b"ab"[..].into());
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.ok().unwrap().unwrap(), b"value".to_vec());
    }

    #[test]
    fn test_iter() {
        let core_db1 = GlobalMemoryDb::open("test1");
        let test1_root: Bucket = core_db1.root();
        let test1_bucket_a: Bucket = test1_root.bucket(&"a".into());
        assert_eq!(test1_bucket_a.put(&"5".into(), &b"value"[..]).is_ok(), true);
        let core_db2 = GlobalMemoryDb::open("test2");
        let root: Bucket = core_db2.root();
        let bucket_a: Bucket = root.bucket(&"a".into());
        let bucket_b: Bucket = root.bucket(&"b".into());
        assert_eq!(bucket_a.put(&"0".into(), &b"value"[..]).is_ok(), true);
        assert_eq!(bucket_a.put(&"1".into(), &b"value"[..]).is_ok(), true);
        assert_eq!(bucket_b.put(&"0".into(), &b"value"[..]).is_ok(), true);
        let result = bucket_a.iter_all(&"".into(), Box::new(move |_k, _v| true));
        assert_eq!(result.len(), 2);
    }
}
