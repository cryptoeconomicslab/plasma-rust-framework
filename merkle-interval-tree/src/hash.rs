use bytes::Bytes;
use sha3::{Digest, Keccak256};

pub fn hash_leaf(value: &Bytes) -> Bytes {
    let mut hasher = Keccak256::new();
    hasher.input(value.as_ref());
    let result = hasher.result();
    Bytes::from(result.as_slice())
}

#[cfg(test)]
mod tests {
    use super::hash_leaf;
    use bytes::Bytes;

    #[test]
    fn test_hash_leaf() {
        let message = Bytes::from("message");
        let hashed = hash_leaf(&message);
        assert_eq!(
            hex::encode(hashed.as_ref()),
            "c2baf6c66618acd49fb133cebc22f55bd907fe9f0d69a726d45b7539ba6bbe08"
        );
    }
}
