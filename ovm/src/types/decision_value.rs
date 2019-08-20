use crate::types::Witness;
use ethabi::{ParamType, Token};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};

pub struct DecisionValue {
    decision: bool,
    witness: Witness,
}

impl DecisionValue {
    pub fn new(decision: bool, witness: Witness) -> Self {
        Self { decision, witness }
    }
    pub fn get_decision(&self) -> bool {
        self.decision
    }
    pub fn get_witness(&self) -> &Witness {
        &self.witness
    }
}

impl Encodable for DecisionValue {
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

impl Decodable for DecisionValue {
    type Ok = DecisionValue;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let decision = tuple[0].clone().to_bool();
        let witness = tuple[1].clone().to_bytes();
        if let (Some(decision), Some(witness)) = (decision, witness) {
            Ok(DecisionValue::new(
                decision,
                Witness::from_abi(&witness).unwrap(),
            ))
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
