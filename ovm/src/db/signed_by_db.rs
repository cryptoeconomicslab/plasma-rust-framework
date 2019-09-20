use crate::error::Error;
use crate::utils::static_hash;
use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_db::{BaseDbKey, KeyValueStore};

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct SignedByRecord {
    pub public_key: Address,
    pub message: Bytes,
    pub signature: Bytes,
}

impl SignedByRecord {
    pub fn new(public_key: Address, message: Bytes, signature: Bytes) -> Self {
        Self {
            public_key,
            message,
            signature,
        }
    }
}

pub struct SignedByDb<'a, KVS: KeyValueStore> {
    db: &'a KVS,
}

impl<'a, KVS: KeyValueStore> SignedByDb<'a, KVS> {
    pub fn new(db: &'a KVS) -> Self {
        Self { db }
    }
    pub fn store_witness(
        &self,
        public_key: Address,
        message: Bytes,
        signature: Bytes,
    ) -> Result<(), Error> {
        self.db
            .bucket(&Bytes::from("signed_by_decider").into())
            .bucket(&BaseDbKey::from(public_key.as_bytes()))
            .put(
                &BaseDbKey::from(static_hash(&message).as_bytes()),
                &SignedByRecord::new(public_key, message, signature).to_abi(),
            )
            .map_err::<Error, _>(Into::into)
    }
    pub fn get_witness(
        &self,
        public_key: Address,
        message: &Bytes,
    ) -> Result<SignedByRecord, Error> {
        let result = self
            .db
            .bucket(&Bytes::from("signed_by_decider").into())
            .bucket(&BaseDbKey::from(public_key.as_bytes()))
            .get(&BaseDbKey::from(static_hash(message).as_bytes()))
            .map_err::<Error, _>(Into::into)?;
        if result.is_none() {
            panic!("signature not found");
        }
        SignedByRecord::from_abi(&result.unwrap()).map_err::<Error, _>(Into::into)
    }
    pub fn get_all_signed_by(&self, signer: Address) -> Vec<SignedByRecord> {
        self.db
            .bucket(&Bytes::from("signed_by_decider").into())
            .bucket(&signer.as_bytes().into())
            .iter_all(&Bytes::default().into(), Box::new(move |_k, _v| true))
            .iter()
            .filter_map(|kv| SignedByRecord::from_abi(kv.get_value()).ok())
            .collect()
    }
}

// require to implement common interface "handle_message"
