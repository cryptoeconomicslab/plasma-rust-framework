use bytes::Bytes;
use ethereum_types::H256;
use tiny_keccak::Keccak;

pub fn static_hash(preimage: &Bytes) -> H256 {
    let mut sha3 = Keccak::new_sha3_256();

    sha3.update(preimage.as_ref());

    let mut res: [u8; 32] = [0; 32];
    sha3.finalize(&mut res);
    H256::from(res)
}
