use crate::error::Error;
use crate::types::Integer;
use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::{Decodable, Encodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use plasma_db::traits::kvs::KeyValueStore;

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct Channel {
    pub channel_id: Bytes,
    pub exitted: Integer,
}

impl Channel {
    pub fn new(channel_id: Bytes, exitted: Integer) -> Self {
        Self {
            channel_id,
            exitted,
        }
    }
}

pub struct ChannelDb<'a, KVS> {
    db: &'a KVS,
}

impl<'a, KVS> ChannelDb<'a, KVS>
where
    KVS: KeyValueStore,
{
    pub fn mark_exited(&self, channel_id: &Bytes) -> Result<(), Error> {
        self.db
            .bucket(&Bytes::from("channels").into())
            .put(
                &channel_id.into(),
                &Channel::new(channel_id.clone(), Integer(1)).to_abi(),
            )
            .map_err(Into::into)
    }
    pub fn get_exited(&self, channel_id: &Bytes) -> Result<Channel, Error> {
        let result = self
            .db
            .bucket(&Bytes::from("channels").into())
            .get(&channel_id.into())
            .map_err::<Error, _>(Into::into)?;
        Channel::from_abi(&result.unwrap()).map_err(Into::into)
    }
}

impl<'a, KVS> From<&'a KVS> for ChannelDb<'a, KVS>
where
    KVS: KeyValueStore,
{
    fn from(kvs: &'a KVS) -> Self {
        Self { db: kvs }
    }
}
