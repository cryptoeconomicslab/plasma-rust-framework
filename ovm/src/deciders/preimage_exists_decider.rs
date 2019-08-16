use crate::db::HashPreimageDb;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{
    Decider, Decision, DecisionValue, ImplicationProofElement, PreimageExistsInput, Property,
    Witness,
};
use bytes::Bytes;
use ethereum_types::H256;
use plasma_core::data_structure::abi::Decodable;
use plasma_db::traits::kvs::{BaseDbKey, KeyValueStore};
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
    type Input = PreimageExistsInput;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &PreimageExistsInput,
        _witness: Option<Witness>,
    ) -> Result<Decision, Error> {
        let key = input.get_hash();
        let db: HashPreimageDb<T> = HashPreimageDb::new(decider.get_db());
        let witness = db.get_witness(key)?;
        if let Witness::Bytes(preimage) = witness {
            if Verifier::hash(&preimage) != input.get_hash() {
                return Err(Error::from(ErrorKind::InvalidPreimage));
            }
            Ok(Decision::new(
                true,
                vec![ImplicationProofElement::new(
                    Property::PreimageExistsDecider(input.clone()),
                    Some(Witness::Bytes(preimage)),
                )],
            ))
        } else {
            panic!("invalid witness")
        }
    }
    fn check_decision<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &PreimageExistsInput,
    ) -> Result<Decision, Error> {
        let decision_key = input.get_hash();
        let result = decider
            .get_db()
            .bucket(&BaseDbKey::from(&b"preimage_exists_decider"[..]))
            .get(&BaseDbKey::from(decision_key.as_bytes()))
            .map_err::<Error, _>(Into::into)?;
        if let Some(decision_value_bytes) = result {
            let decision_value = DecisionValue::from_abi(&decision_value_bytes).unwrap();
            return Ok(Decision::new(
                decision_value.get_decision(),
                vec![ImplicationProofElement::new(
                    Property::PreimageExistsDecider(input.clone()),
                    Some(decision_value.get_witness().clone()),
                )],
            ));
        }

        Err(Error::from(ErrorKind::Undecided))
    }
}

#[cfg(test)]
mod tests {
    use crate::db::HashPreimageDb;
    use crate::deciders::preimage_exists_decider::Verifier;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{Decision, PreimageExistsInput, Property, Witness};
    use bytes::Bytes;
    use plasma_db::impls::kvs::CoreDbLevelDbImpl;

    #[test]
    fn test_decide() {
        let preimage = Bytes::from("left");
        let hash = Verifier::static_hash(&preimage);
        let input = PreimageExistsInput::new(hash);
        let property = Property::PreimageExistsDecider(input.clone());
        let witness = Witness::Bytes(preimage);
        let decider: PropertyExecutor<CoreDbLevelDbImpl> = Default::default();
        let db = HashPreimageDb::new(decider.get_db());
        assert!(db.store_witness(hash, &witness).is_ok());
        let decided: Decision = decider.decide(&property, Some(witness)).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }

}
