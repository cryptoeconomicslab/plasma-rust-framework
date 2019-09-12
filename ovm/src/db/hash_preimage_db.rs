use crate::error::Error;
use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::H256;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_db::traits::kvs::{BaseDbKey, KeyValueStore};

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct PreimageRecord {
    pub hash: H256,
    pub preimage: Bytes,
}

impl PreimageRecord {
    pub fn new(hash: H256, preimage: Bytes) -> Self {
        Self { hash, preimage }
    }
}

pub struct HashPreimageDb<'a, KVS: KeyValueStore> {
    db: &'a KVS,
}

impl<'a, KVS: KeyValueStore> HashPreimageDb<'a, KVS> {
    pub fn new(db: &'a KVS) -> Self {
        Self { db }
    }
    pub fn store_witness(&self, hash: H256, preimage: &Bytes) -> Result<(), Error> {
        let record = PreimageRecord::new(hash, preimage.clone());
        self.db
            .bucket(&BaseDbKey::from(&b"preimage_exists_decider"[..]))
            .put(&BaseDbKey::from(hash.as_bytes()), &record.to_abi())
            .map_err::<Error, _>(Into::into)
    }
    pub fn get_witness(&self, hash: H256) -> Result<PreimageRecord, Error> {
        let result = self
            .db
            .bucket(&BaseDbKey::from(&b"preimage_exists_decider"[..]))
            .get(&BaseDbKey::from(hash.as_bytes()))
            .map_err::<Error, _>(Into::into)?;
        if result.is_none() {
            panic!("preimage not found");
        }
        PreimageRecord::from_abi(&result.unwrap()).map_err::<Error, _>(Into::into)
    }
}

// require to implement common interface "handle_message"
