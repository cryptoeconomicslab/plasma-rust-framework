use crate::error::Error;
use crate::types::Integer;
use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::{Decodable, Encodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use plasma_db::traits::kvs::KeyValueStore;

#[derive(Clone, Debug, PartialEq, Eq, AbiDecodable, AbiEncodable)]
pub struct Message {
    pub channel_id: Bytes,
    pub nonce: Integer,
    message: Bytes,
}

impl Message {
    pub fn new(channel_id: Bytes, nonce: Integer, message: Bytes) -> Self {
        Self {
            channel_id,
            nonce,
            message,
        }
    }
}

pub struct MessageDb<'a, KVS> {
    db: &'a KVS,
}

impl<'a, KVS> MessageDb<'a, KVS>
where
    KVS: KeyValueStore,
{
    pub fn get_message_by_channel_id_and_nonce(
        &self,
        channel_id: Bytes,
        nonce: Integer,
    ) -> Option<Message> {
        let nonce_bytes: Bytes = nonce.into();
        self.db
            .bucket(&channel_id.into())
            .get(&nonce_bytes.into())
            .ok()
            .unwrap()
            .map(|b| Message::from_abi(&b).ok().unwrap())
    }
    pub fn get_most_recent_message(&self, channel_id: &Bytes) -> Option<Message> {
        let mut list: Vec<Message> = self
            .db
            .bucket(&channel_id.into())
            .iter_all(&Bytes::from("").into(), Box::new(move |_k, _v| true))
            .iter()
            .filter_map(|kv| Message::from_abi(kv.get_value()).ok())
            .collect();
        list.pop()
    }
    pub fn store_message(&self, message: &Message) -> Result<(), Error> {
        let nonce_bytes: Bytes = message.nonce.into();
        let channel_id: Bytes = message.channel_id.clone();
        self.db
            .bucket(&channel_id.into())
            .put(&nonce_bytes.into(), &message.to_abi())
            .map_err(Into::into)
    }
}

impl<'a, KVS> From<&'a KVS> for MessageDb<'a, KVS>
where
    KVS: KeyValueStore,
{
    fn from(kvs: &'a KVS) -> Self {
        Self { db: kvs }
    }
}
