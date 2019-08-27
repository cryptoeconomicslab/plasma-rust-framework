use crate::types::Integer;
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};
use plasma_db::traits::kvs::KeyValueStore;

#[derive(Clone, Debug)]
pub struct Message {
    pub channel_id: Bytes,
    sender: Address,
    recipient: Address,
    pub nonce: Integer,
    signers: Vec<Address>,
    message: Bytes,
    signed_message: Bytes,
}

impl Message {
    pub fn new(
        channel_id: Bytes,
        sender: Address,
        recipient: Address,
        nonce: Integer,
        message: Bytes,
    ) -> Self {
        Self {
            channel_id,
            sender,
            recipient,
            nonce,
            signers: vec![],
            message,
            signed_message: Bytes::from(""),
        }
    }
    pub fn get_signers(&self) -> &Vec<Address> {
        &self.signers
    }
}

impl Encodable for Message {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Bytes(self.channel_id.to_vec()),
            Token::Address(self.sender),
            Token::Address(self.recipient),
            Token::Uint(self.nonce.0.into()),
            Token::Array(self.signers.iter().map(|s| Token::Address(*s)).collect()),
            Token::Bytes(self.message.to_vec()),
            Token::Bytes(self.signed_message.to_vec()),
        ]
    }
}

impl Decodable for Message {
    type Ok = Message;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let channel_id = tuple[0].clone().to_bytes();
        let sender = tuple[1].clone().to_address();
        let recipient = tuple[2].clone().to_address();
        let nonce = tuple[3].clone().to_uint();
        let signers = tuple[4].clone().to_array();
        let message = tuple[5].clone().to_bytes();
        let signed_message = tuple[6].clone().to_bytes();
        if let (
            Some(channel_id),
            Some(sender),
            Some(recipient),
            Some(nonce),
            Some(signers),
            Some(message),
            Some(signed_message),
        ) = (
            channel_id,
            sender,
            recipient,
            nonce,
            signers,
            message,
            signed_message,
        ) {
            Ok(Message {
                channel_id: Bytes::from(channel_id),
                sender,
                recipient,
                nonce: Integer(nonce.as_u64()),
                signers: signers
                    .iter()
                    .map(|s| s.clone().to_address().unwrap())
                    .collect(),
                message: Bytes::from(message),
                signed_message: Bytes::from(signed_message),
            })
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, PlasmaCoreError> {
        let decoded = ethabi::decode(
            &[
                ParamType::Bytes,
                ParamType::Address,
                ParamType::Address,
                ParamType::Uint(256),
                ParamType::Array(Box::new(ParamType::Address)),
                ParamType::Bytes,
                ParamType::Bytes,
            ],
            data,
        )
        .map_err(|_e| PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

pub struct MessageDb<KVS> {
    db: KVS,
}

impl<KVS> MessageDb<KVS>
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
    pub fn get_messages_signed_by(
        &self,
        signer: Address,
        _channel_id: Option<Bytes>,
        _nonce: Option<Integer>,
    ) -> Vec<Message> {
        self.db
            .iter_all(&Bytes::from("").into(), Box::new(move |_k, _v| true))
            .iter()
            .filter_map(|kv| Message::from_abi(kv.get_value()).ok())
            .filter(|message| message.get_signers().contains(&signer))
            .collect()
    }
}

impl<KVS> From<KVS> for MessageDb<KVS>
where
    KVS: KeyValueStore,
{
    fn from(kvs: KVS) -> Self {
        Self { db: kvs }
    }
}
