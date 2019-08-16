use crate::error::Error;
use crate::types::Witness;
use ethereum_types::H256;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_db::traits::kvs::{BaseDbKey, KeyValueStore};

pub struct HashPreimageDb<'a, KVS: KeyValueStore> {
    db: &'a KVS,
}

impl<'a, KVS: KeyValueStore> HashPreimageDb<'a, KVS> {
    pub fn new(db: &'a KVS) -> Self {
        Self { db }
    }
    pub fn store_witness(&self, hash: H256, preimage: &Witness) -> Result<(), Error> {
        self.db
            .bucket(&BaseDbKey::from(&b"preimage_exists_decider"[..]))
            .put(&BaseDbKey::from(hash.as_bytes()), &preimage.to_abi())
            .map_err::<Error, _>(Into::into)
    }
    pub fn get_witness(&self, hash: H256) -> Result<Witness, Error> {
        let result = self
            .db
            .bucket(&BaseDbKey::from(&b"preimage_exists_decider"[..]))
            .get(&BaseDbKey::from(hash.as_bytes()))
            .map_err::<Error, _>(Into::into)?;
        if result.is_none() {
            panic!("preimage not found");
        }
        Witness::from_abi(&result.unwrap()).map_err::<Error, _>(Into::into)
    }
}

// require to implement common interface "handle_message"
