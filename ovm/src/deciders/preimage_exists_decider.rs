use crate::error::{Error, ErrorKind};
use crate::property_executer::PropertyExecuter;
use crate::types::{Decider, Decision, ImplicationProofElement, PreimageExistsInput, Property};
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

pub struct PreimageExistsDecisionValue {
    decision: bool,
    witness: Bytes,
}

impl PreimageExistsDecisionValue {
    pub fn new(decision: bool, witness: Bytes) -> Self {
        PreimageExistsDecisionValue { decision, witness }
    }
    pub fn get_decision(&self) -> bool {
        self.decision
    }
    pub fn get_witness(&self) -> &Bytes {
        &self.witness
    }
}

impl Encodable for PreimageExistsDecisionValue {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Bool(self.decision),
            Token::Bytes(self.witness.to_vec()),
        ]
    }
}

impl Decodable for PreimageExistsDecisionValue {
    type Ok = PreimageExistsDecisionValue;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let decision = tuple[0].clone().to_bool();
        let witness = tuple[1].clone().to_bytes();
        if let (Some(decision), Some(witness)) = (decision, witness) {
            Ok(PreimageExistsDecisionValue::new(decision, witness.into()))
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, PlasmaCoreError> {
        let decoded = ethabi::decode(&[ParamType::Bool, ParamType::Bytes], data)
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
    fn decide(
        decider: &PropertyExecuter,
        input: &PreimageExistsInput,
        witness_bytes: Option<&Bytes>,
    ) -> Result<Decision, Error> {
        let preimage = witness_bytes.unwrap();

        if Verifier::hash(preimage) != input.get_hash() {
            return Err(Error::from(ErrorKind::InvalidPreimage));
        }

        let decision_key = input.get_hash();
        let decision_value = PreimageExistsDecisionValue::new(true, preimage.clone());
        decider
            .get_db()
            .bucket(&BaseDbKey::from(&b"preimage_exists_decider"[..]))
            .put(
                &BaseDbKey::from(decision_key.as_bytes()),
                &decision_value.to_abi(),
            )
            .map_err::<Error, _>(Into::into)?;

        Ok(Decision::new(true, vec![]))
    }
    fn check_decision(
        decider: &PropertyExecuter,
        input: &PreimageExistsInput,
    ) -> Result<Decision, Error> {
        let decision_key = input.get_hash();
        let result = decider
            .get_db()
            .bucket(&BaseDbKey::from(&b"preimage_exists_decider"[..]))
            .get(&BaseDbKey::from(decision_key.as_bytes()))
            .map_err::<Error, _>(Into::into)?;
        if let Some(decision_value_bytes) = result {
            let decision_value =
                PreimageExistsDecisionValue::from_abi(&decision_value_bytes).unwrap();
            return Ok(Decision::new(
                decision_value.get_decision(),
                vec![ImplicationProofElement::new(
                    Property::PreimageExistsDecider(Box::new(input.clone())),
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
    use crate::property_executer::PropertyExecuter;
    use crate::types::{Decider, Decision, PreimageExistsInput, Property};
    use bytes::Bytes;

    #[test]
    fn test_decide() {
        let input = PreimageExistsInput::new(Verifier::static_hash(&Bytes::from("left")));
        let property = Property::PreimageExistsDecider(Box::new(input.clone()));
        let witness = Bytes::from("left");
        let decider: PropertyExecuter = Default::default();
        let decided: Decision = decider.decide(&property, Some(&witness)).unwrap();
        assert_eq!(decided.get_outcome(), true);
        let status = PreimageExistsDecider::check_decision(&decider, &input).unwrap();
        assert_eq!(status.get_outcome(), true);
    }

}
