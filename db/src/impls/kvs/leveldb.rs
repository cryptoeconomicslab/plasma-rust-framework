use crate::error::Error;
use crate::traits::db::DatabaseTrait;
use crate::traits::kvs::{BaseDbKey, Batch, Bucket, KeyValue, KeyValueStore};
use db_key::Key;
use leveldb::batch::Batch as LevelBatch;
use leveldb::database::kv::KV;
use leveldb::database::{batch::Writebatch, Database};
use leveldb::iterator::Iterable;
use leveldb::iterator::LevelDBIterator;
use leveldb::options::{Options, ReadOptions, WriteOptions};
use parking_lot::RwLock;
use std::path::Path;
use tempdir::TempDir;

impl Key for BaseDbKey {
    fn from_u8(key: &[u8]) -> BaseDbKey {
        BaseDbKey::new(key.to_vec())
    }

    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(&self.as_bytes())
    }
}

pub struct CoreDb {
    db: RwLock<Database<BaseDbKey>>,
}

impl CoreDb {
    pub fn open_with_tempdir(dbname: &str) -> Self {
        let tempdir = TempDir::new(dbname).unwrap();
        Self::open_with_path(tempdir.path())
    }
    pub fn open_with_path(path: &Path) -> Self {
        let mut leveldb_options = Options::new();
        leveldb_options.create_if_missing = true;
        Self {
            db: RwLock::new(Database::open(path, leveldb_options).unwrap()),
        }
    }
}

impl DatabaseTrait for CoreDb {
    fn open(dbname: &str) -> Self {
        let path = Path::new("./.plasma_db").join(dbname);
        Self::open_with_path(path.as_path())
    }
    fn close(&self) {}
}

impl KeyValueStore for CoreDb {
    fn get(&self, key: &BaseDbKey) -> Result<Option<Vec<u8>>, Error> {
        let read_opts = ReadOptions::new();
        self.db
            .read()
            .get(read_opts, key)
            .map_err(Into::into)
            .map(|v| v.map(|v| v.to_vec()))
    }
    fn put(&self, key: &BaseDbKey, value: &[u8]) -> Result<(), Error> {
        let write_opts = WriteOptions::new();
        self.db
            .write()
            .put(write_opts, key, value)
            .map_err(Into::into)
    }
    fn del(&self, key: &BaseDbKey) -> Result<(), Error> {
        let write_opts = WriteOptions::new();
        self.db.write().delete(write_opts, key).map_err(Into::into)
    }
    fn has(&self, _key: &BaseDbKey) -> Result<bool, Error> {
        Ok(true)
    }
    fn batch(&self, operations: &[Batch]) -> Result<(), Error> {
        let mut batch: Writebatch<BaseDbKey> = Writebatch::new();
        for op in operations.iter() {
            match op {
                Batch::BatchPut { key, value } => {
                    batch.put(key.clone(), value);
                }
                Batch::BatchDel { key } => batch.delete(key.clone()),
            }
        }
        self.db
            .write()
            .write(WriteOptions::new(), &batch)
            .map_err(Into::into)
    }
    fn iter_all_with_prefix(
        &self,
        prefix: &BaseDbKey,
        start: &BaseDbKey,
        mut f: Box<dyn FnMut(&BaseDbKey, &Vec<u8>) -> bool>,
    ) -> Vec<KeyValue> {
        let read_lock = self.db.read();
        let iter = read_lock.iter(ReadOptions::new());
        let mut result = vec![];
        iter.seek(&prefix.concat(start));
        for (k, v) in iter {
            if prefix.concat(start) == k {
                continue;
            }
            if k.0.starts_with(&prefix.0) && f(&k, &v) {
                result.push(KeyValue::new(k.clone(), v.clone()));
                continue;
            } else {
                break;
            }
        }
        result
    }
    fn iter_all(
        &self,
        start: &BaseDbKey,
        mut f: Box<dyn FnMut(&BaseDbKey, &Vec<u8>) -> bool>,
    ) -> Vec<KeyValue> {
        let read_lock = self.db.read();
        let iter = read_lock.iter(ReadOptions::new());
        let mut result = vec![];
        iter.seek(start);
        for (k, v) in iter {
            if *start == k {
                continue;
            }
            if f(&k, &v) {
                result.push(KeyValue::new(k.clone(), v.clone()));
                continue;
            } else {
                break;
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
    use super::CoreDb;
    use crate::traits::kvs::{Bucket, KeyValueStore};

    #[test]
    fn test_bucket() {
        let core_db = CoreDb::open_with_tempdir("test");
        let root: Bucket = core_db.root();
        let bucket: Bucket = root.bucket(&b"a"[..].into());
        assert_eq!(bucket.put(&b"b"[..].into(), &b"value"[..]).is_ok(), true);
        let result = root.get(&b"ab"[..].into());
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.ok().unwrap().unwrap(), b"value".to_vec());
    }

    #[test]
    fn test_iter() {
        let core_db = CoreDb::open_with_tempdir("test");
        let root: Bucket = core_db.root();
        let bucket_a: Bucket = root.bucket(&"a".into());
        let bucket_b: Bucket = root.bucket(&"b".into());
        assert_eq!(bucket_a.put(&"0".into(), &b"value"[..]).is_ok(), true);
        assert_eq!(bucket_a.put(&"1".into(), &b"value"[..]).is_ok(), true);
        assert_eq!(bucket_b.put(&"0".into(), &b"value"[..]).is_ok(), true);
        let result = bucket_a.iter_all(&"".into(), Box::new(move |_k, _v| true));
        assert_eq!(result.len(), 2);
    }
}
