use crate::db::HashPreimageDb;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, ImplicationProofElement, PropertyInput};
use crate::DeciderManager;
use bytes::Bytes;
use ethereum_types::H256;
use plasma_db::traits::kvs::KeyValueStore;
use tiny_keccak::Keccak;

pub struct Verifier {}

impl Verifier {
    pub fn hash(preimage: &Bytes) -> H256 {
        Self::static_hash(preimage)
    }
    pub fn static_hash(preimage: &Bytes) -> H256 {
        let mut sha3 = Keccak::new_sha3_256();

        sha3.update(preimage.as_ref());

        let mut res: [u8; 32] = [0; 32];
        sha3.finalize(&mut res);
        H256::from(res)
    }
}

pub struct PreimageExistsDecider {}

impl Default for PreimageExistsDecider {
    fn default() -> Self {
        PreimageExistsDecider {}
    }
}

impl Decider for PreimageExistsDecider {
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        inputs: &[PropertyInput],
    ) -> Result<Decision, Error> {
        let hash = decider.get_variable(&inputs[0]).to_h256();
        let key = hash;
        let db: HashPreimageDb<T> = HashPreimageDb::new(decider.get_db());
        let preimage_record = db.get_witness(key)?;
        if Verifier::hash(&preimage_record.preimage) != hash {
            return Err(Error::from(ErrorKind::InvalidPreimage));
        }
        Ok(Decision::new(
            true,
            vec![ImplicationProofElement::new(
                DeciderManager::preimage_exists_decider(inputs.to_vec()),
                Some(preimage_record.preimage),
            )],
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::db::HashPreimageDb;
    use crate::deciders::preimage_exists_decider::Verifier;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{Decision, PropertyInput};
    use crate::DeciderManager;
    use bytes::Bytes;
    use plasma_db::impls::kvs::CoreDbMemoryImpl;

    #[test]
    fn test_decide() {
        let preimage = Bytes::from("left");
        let hash = Verifier::static_hash(&preimage);
        let property =
            DeciderManager::preimage_exists_decider(vec![PropertyInput::ConstantH256(hash)]);
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        let db = HashPreimageDb::new(decider.get_db());
        assert!(db.store_witness(hash, &preimage).is_ok());
        let decided: Decision = decider.decide(&property).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }
}
