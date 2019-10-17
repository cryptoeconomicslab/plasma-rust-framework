use bytes::Bytes;
use sha3::{Digest, Sha3_256};

pub fn hash_leaf(value: &Bytes) -> Bytes {
    let mut hasher = Sha3_256::new();
    hasher.input(value.as_ref());
    let result = hasher.result();
    Bytes::from(result.as_slice())
}
