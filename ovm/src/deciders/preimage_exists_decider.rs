use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{
    Decider, Decision, DecisionValue, ImplicationProofElement, PreimageExistsInput, Property,
    Witness,
};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::H256;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};
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

#[derive(Clone, Debug)]
pub struct PreimageExistsWitness {
    preimage: Bytes,
}

impl PreimageExistsWitness {
    pub fn new(preimage: Bytes) -> Self {
        PreimageExistsWitness { preimage }
    }
}

impl Encodable for PreimageExistsWitness {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![Token::Bytes(self.preimage.to_vec())]
    }
}

impl Decodable for PreimageExistsWitness {
    type Ok = PreimageExistsWitness;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let preimage = tuple[0].clone().to_bytes();
        if let Some(preimage) = preimage {
            Ok(PreimageExistsWitness::new(Bytes::from(preimage)))
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, PlasmaCoreError> {
        let decoded = ethabi::decode(&[ParamType::Bytes], data)
            .map_err(|_e| PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
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
        witness: Option<Witness>,
    ) -> Result<Decision, Error> {
        if let Some(Witness::Bytes(preimage)) = witness {
            if Verifier::hash(&preimage) != input.get_hash() {
                return Err(Error::from(ErrorKind::InvalidPreimage));
            }

            let decision_key = input.get_hash();
            let decision_value = DecisionValue::new(true, Witness::Bytes(preimage.clone()));
            decider
                .get_db()
                .bucket(&BaseDbKey::from(&b"preimage_exists_decider"[..]))
                .put(
                    &BaseDbKey::from(decision_key.as_bytes()),
                    &decision_value.to_abi(),
                )
                .map_err::<Error, _>(Into::into)?;

            Ok(Decision::new(true, vec![]))
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
    use super::PreimageExistsDecider;
    use crate::deciders::preimage_exists_decider::Verifier;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{Decider, Decision, PreimageExistsInput, Property, Witness};
    use bytes::Bytes;
    use plasma_db::impls::kvs::CoreDbLevelDbImpl;

    #[test]
    fn test_decide() {
        let input = PreimageExistsInput::new(Verifier::static_hash(&Bytes::from("left")));
        let property = Property::PreimageExistsDecider(input.clone());
        let witness = Witness::Bytes(Bytes::from("left"));
        let decider: PropertyExecutor<CoreDbLevelDbImpl> = Default::default();
        let decided: Decision = decider.decide(&property, Some(witness)).unwrap();
        assert_eq!(decided.get_outcome(), true);
        let status = PreimageExistsDecider::check_decision(&decider, &input).unwrap();
        assert_eq!(status.get_outcome(), true);
    }

}
