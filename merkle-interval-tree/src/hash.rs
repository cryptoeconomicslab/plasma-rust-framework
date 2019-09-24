use bytes::Bytes;
use crypto::digest::Digest;
use crypto::sha3::Sha3;

pub fn hash_leaf(value: &Bytes) -> Bytes {
    let mut hasher = Sha3::keccak256();
    let mut result = vec![0u8; hasher.output_bits() / 8];
    hasher.reset();
    hasher.input(value.as_ref());
    hasher.result(result.as_mut_slice());
    Bytes::from(result)
}
