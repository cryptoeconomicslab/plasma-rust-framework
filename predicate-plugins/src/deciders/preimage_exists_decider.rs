use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::{Address, H256};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{Error, ErrorKind};
use plasma_core::ovm::{
    Decider, Decision, DecisionStatus, ImplicationProofElement, Property, Witness,
};
use plasma_db::impls::kvs::CoreDbLevelDbImpl;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::{BaseDbKey, KeyValueStore};
use tiny_keccak::Keccak;

pub struct Verifier {}

impl Default for Verifier {
    fn default() -> Self {
        Verifier {}
    }
}

impl Verifier {
    pub fn hash(&self, preimage: &Bytes) -> H256 {
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

pub struct PreimageExistsInput {
    verifier: Verifier,
    parameters: Bytes,
    hash: H256,
}

impl PreimageExistsInput {
    pub fn new(verifier: Verifier, parameters: Bytes, hash: H256) -> Self {
        PreimageExistsInput {
            verifier,
            parameters,
            hash,
        }
    }
    pub fn get_parameters(&self) -> &Bytes {
        &self.parameters
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

impl Witness for PreimageExistsWitness {}

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
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let preimage = tuple[0].clone().to_bytes();
        if let Some(preimage) = preimage {
            Ok(PreimageExistsWitness::new(Bytes::from(preimage)))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded = ethabi::decode(&[ParamType::Bytes], data)
            .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

pub struct PreimageExistsDecisionValue {
    decision: bool,
    witness: PreimageExistsWitness,
}

impl PreimageExistsDecisionValue {
    pub fn new(decision: bool, witness: PreimageExistsWitness) -> Self {
        PreimageExistsDecisionValue { decision, witness }
    }
    pub fn get_decision(&self) -> bool {
        self.decision
    }
    pub fn get_witness(&self) -> &PreimageExistsWitness {
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
            Token::Bytes(self.witness.to_abi()),
        ]
    }
}

impl Decodable for PreimageExistsDecisionValue {
    type Ok = PreimageExistsDecisionValue;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let decision = tuple[0].clone().to_bool();
        let witness = tuple[1].clone().to_bytes();
        if let (Some(decision), Some(witness)) = (decision, witness) {
            Ok(PreimageExistsDecisionValue::new(
                decision,
                PreimageExistsWitness::from_abi(&witness).unwrap(),
            ))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded = ethabi::decode(&[ParamType::Bool, ParamType::Bytes], data)
            .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

pub struct PreimageExistsDecider {
    db: CoreDbLevelDbImpl,
}

impl Default for PreimageExistsDecider {
    fn default() -> Self {
        PreimageExistsDecider {
            db: CoreDbLevelDbImpl::open("test"),
        }
    }
}

impl Decider for PreimageExistsDecider {
    type Input = PreimageExistsInput;
    type Witness = PreimageExistsWitness;

    fn decide(&self, input: &PreimageExistsInput, witness: PreimageExistsWitness) -> Decision {
        let preimage = &witness.preimage;

        if input.verifier.hash(preimage) != input.hash {
            panic!("invalid preimage")
        }

        let decision_key = input.hash;
        let decision_value = PreimageExistsDecisionValue::new(true, witness.clone());
        if self
            .db
            .bucket(&BaseDbKey::from(&b"preimage_exists_decider"[..]))
            .put(
                &BaseDbKey::from(decision_key.as_bytes()),
                &decision_value.to_abi(),
            )
            .is_err()
        {
            panic!("failed to store data")
        }

        Decision::new(DecisionStatus::Decided(true), vec![])
    }
    fn check_decision(&self, input: &PreimageExistsInput) -> Decision {
        let decision_key = input.hash;
        let result = self
            .db
            .bucket(&BaseDbKey::from(&b"preimage_exists_decider"[..]))
            .get(&BaseDbKey::from(decision_key.as_bytes()));
        if let Ok(Some(decision_value_bytes)) = result {
            let decision_value =
                PreimageExistsDecisionValue::from_abi(&decision_value_bytes).unwrap();
            return Decision::new(
                DecisionStatus::Decided(decision_value.get_decision()),
                vec![ImplicationProofElement::new(
                    Property::new(Address::zero(), input.get_parameters().clone()),
                    Bytes::from(decision_value.get_witness().to_abi()),
                )],
            );
        }

        Decision::new(DecisionStatus::Undecided, vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Decision, DecisionStatus, PreimageExistsDecider, PreimageExistsInput,
        PreimageExistsWitness, Verifier,
    };
    use bytes::Bytes;
    use plasma_core::ovm::Decider;

    #[test]
    fn test_decide() {
        let preimage_exists_decider: PreimageExistsDecider = Default::default();
        let input = PreimageExistsInput::new(
            Default::default(),
            Bytes::from(""),
            Verifier::static_hash(&Bytes::from("test")),
        );
        let decided: Decision =
            preimage_exists_decider.decide(&input, PreimageExistsWitness::new(Bytes::from("test")));
        assert_eq!(decided.get_outcome(), &DecisionStatus::Decided(true));
        let status = preimage_exists_decider.check_decision(&input);
        assert_eq!(status.get_outcome(), &DecisionStatus::Decided(true));
    }

}
