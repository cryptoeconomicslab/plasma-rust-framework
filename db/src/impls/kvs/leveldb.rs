use crate::error::{Error, ErrorKind};
use crate::traits::db::DatabaseTrait;
use crate::traits::kvs::{BaseDbKey, Batch, Bucket, KeyValueStore, KvsIterator};
use db_key::Key;
use leveldb::database::kv::KV;
use leveldb::database::{batch::Writebatch, Database};
use leveldb::iterator::Iterable;
//use leveldb::kv::KV;
use leveldb::batch::Batch as LevelBatch;
use leveldb::iterator::{LevelDBIterator, ValueIterator};
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

struct CoreDbIterator<'a> {
    iter: &'a Iterator<Item = Vec<u8>>,
}

impl<'a> CoreDbIterator<'a> {
    pub fn new(iter: &'a Iterator<Item = Vec<u8>>) -> Self {
        Self { iter }
    }
}

impl<'a> Iterator for CoreDbIterator<'a> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T> KeyValueStore<T> for CoreDb where T: Iterator<Item = Vec<u8>> {
    fn get(&self, key: &BaseDbKey) -> Result<Option<Box<[u8]>>, Error> {
        let read_opts = ReadOptions::new();
        self.db
            .read()
            .get(read_opts, key)
            .map_err(Into::into)
            .map(|v| v.map(|v| v.to_vec().into_boxed_slice()))
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
    fn iterator(&self, prefix: &BaseDbKey) -> Result<T, Error> {
        // let read_lock = self.db.read();
        let iter = self.db.read().value_iter(ReadOptions::new());
        iter.seek(prefix);
        Ok(iter)
    }
    fn bucket(&self, prefix: &BaseDbKey) -> Box<Bucket<T>> {
        Box::new(Bucket::new(prefix.clone(), self))
    }
}
