use super::error::Error;
use bytes::Bytes;
use plasma_db::traits::kvs::KeyValueStore;

pub struct WalletDb<'a, KVS> {
    db: &'a KVS,
}

impl<'a, KVS: KeyValueStore> WalletDb<'a, KVS> {
    pub fn new(db: &'a KVS) -> Self {
        Self { db }
    }

    pub fn put_private_key(&mut self, key: &Bytes, raw_key: &[u8]) -> Result<(), Error> {
        self.db
            .bucket(&Bytes::from("wallets").into())
            .put(&key.into(), raw_key)
            .map_err::<Error, _>(Into::into)
    }

    pub fn get_private_key(&self, key: &Bytes) -> Result<Vec<u8>, Error> {
        self.db
            .bucket(&Bytes::from("wallets").into())
            .get(&key.into())
            .map(|op| op.unwrap_or_else(|| vec![]))
            .map_err::<Error, _>(Into::into)
    }
}
