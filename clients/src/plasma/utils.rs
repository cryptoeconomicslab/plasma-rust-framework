use bytes::Bytes;
use ethereum_types::Address;

pub fn string_to_address(s: &str) -> Address {
    Address::from_slice(&hex::decode(s).unwrap())
}

pub fn decode_session(session: String) -> Result<Bytes, ()> {
    if let Ok(s) = hex::decode(session) {
        Ok(Bytes::from(s))
    } else {
        Err(())
    }
}

pub fn encode_session(raw: Bytes) -> String {
    hex::encode(raw.to_vec())
}
