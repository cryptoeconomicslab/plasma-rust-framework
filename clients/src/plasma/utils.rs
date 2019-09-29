use bytes::Bytes;
use ethereum_types::Address;

pub fn string_to_address(s: &str) -> Address {
    Address::from_slice(&hex::decode(s).unwrap())
}

pub fn encode_hex(bytes: &Bytes) -> String {
    hex::encode(bytes)
}

pub fn decode_hex(hex_string: String) -> Result<Bytes, ()> {
    if let Ok(s) = hex::decode(hex_string) {
        Ok(Bytes::from(s))
    } else {
        Err(())
    }
}

pub fn decode_session(session: String) -> Result<Bytes, ()> {
    decode_hex(session)
}

pub fn encode_session(raw: Bytes) -> String {
    hex::encode(raw.to_vec())
}
