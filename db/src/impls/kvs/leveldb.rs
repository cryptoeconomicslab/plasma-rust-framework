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
use tempdir::TempDir;

impl Key for BaseDbKey {
    fn from_u8(key: &[u8]) -> BaseDbKey {
        BaseDbKey::new(key.to_vec())
    }

    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(&self.get())
    }
}

pub struct CoreDb {
    db: RwLock<Database<BaseDbKey>>,
}

impl DatabaseTrait for CoreDb {
    fn open(dbname: &str) -> Self {
        let tempdir = TempDir::new(dbname).unwrap();
        let path = tempdir.path();

        let mut options = Options::new();
        options.create_if_missing = true;
        Self {
            db: RwLock::new(Database::open(path, options).unwrap()),
        }
    }
    fn close(&self) {}
}

impl<'a, B> KeyValueStore<B> for CoreDb {
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
                Batch::BatchPut { key, value } => batch.put(key.clone(), value),
                Batch::BatchDel { key } => batch.delete(key.clone()),
            }
        }
        self.db
            .write()
            .write(WriteOptions::new(), &batch)
            .map_err(Into::into)
    }
    fn iter_all(
        &self,
        prefix: &BaseDbKey,
        mut f: Box<FnMut(&BaseDbKey, &Vec<u8>) -> bool>,
    ) -> Vec<KeyValue> {
        let read_lock = self.db.read();
        let iter = read_lock.iter(ReadOptions::new());
        let mut result = vec![];
        iter.seek(prefix);
        for (k, v) in iter {
            if f(&k, &v) {
                result.push(KeyValue::new(k.clone(), v.clone()));
                continue;
            } else {
                break;
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
        let iter = read_lock.iter(ReadOptions::new());
        let mut result = vec![];
        iter.seek(prefix);
        for (k, v) in iter {
            if let Some(b) = f(&k, &v) {
                result.push(b);
                continue;
            } else {
                break;
            }
        }
        result
    }
    fn bucket(&self, prefix: &BaseDbKey) -> Box<Bucket<B>> {
        Box::new(Bucket::new(prefix.clone(), self))
    }
}

#[cfg(test)]
mod tests {
    use super::CoreDb;
    use crate::traits::db::DatabaseTrait;
    use crate::traits::kvs::{Bucket, KeyValueStore};

    #[test]
    fn test_bucket() {
        let core_db = CoreDb::open("test");
        let root: Box<Bucket<Vec<u8>>> = core_db.root();
        let bucket: Box<Bucket<Vec<u8>>> = root.bucket(&b"a"[..].into());
        assert_eq!(bucket.put(&b"b"[..].into(), &b"value"[..]).is_ok(), true);
        let result = root.get(&b"ab"[..].into());
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.ok().unwrap().unwrap(), b"value".to_vec());
    }

}
