use crate::error::Error;
use crate::types::{SignedByInput, Witness};
use crate::utils::static_hash;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_db::traits::kvs::{BaseDbKey, KeyValueStore};

pub struct SignedByDb<'a, KVS: KeyValueStore> {
    db: &'a KVS,
}

impl<'a, KVS: KeyValueStore> SignedByDb<'a, KVS> {
    pub fn new(db: &'a KVS) -> Self {
        Self { db }
    }
    pub fn store_witness(&self, input: &SignedByInput, signature: &Witness) -> Result<(), Error> {
        self.db
            .bucket(&BaseDbKey::from(&b"signed_by_decider"[..]))
            .bucket(&BaseDbKey::from(input.get_public_key().as_bytes()))
            .put(
                &BaseDbKey::from(static_hash(input.get_message()).as_bytes()),
                &signature.to_abi(),
            )
            .map_err::<Error, _>(Into::into)
    }
    pub fn get_witness(&self, input: &SignedByInput) -> Result<Witness, Error> {
        let result = self
            .db
            .bucket(&BaseDbKey::from(&b"signed_by_decider"[..]))
            .bucket(&BaseDbKey::from(input.get_public_key().as_bytes()))
            .get(&BaseDbKey::from(
                static_hash(input.get_message()).as_bytes(),
            ))
            .map_err::<Error, _>(Into::into)?;
        if result.is_none() {
            panic!("signature not found");
        }
        Witness::from_abi(&result.unwrap()).map_err::<Error, _>(Into::into)
    }
}

// require to implement common interface "handle_message"
