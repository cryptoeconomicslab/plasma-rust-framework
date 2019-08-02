use crate::DeciderManager;
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

pub struct AndDeciderInput {
    left: Property,
    left_witness: Bytes,
    right: Property,
    right_witness: Bytes,
}

impl AndDeciderInput {
    pub fn new(left: Property, left_witness: Bytes, right: Property, right_witness: Bytes) -> Self {
        AndDeciderInput {
            left,
            left_witness,
            right,
            right_witness,
        }
    }
    pub fn get_left(&self) -> &Property {
        &self.left
    }
    pub fn get_right(&self) -> &Property {
        &self.right
    }
    pub fn get_left_witness(&self) -> &Bytes {
        &self.left_witness
    }
    pub fn get_right_witness(&self) -> &Bytes {
        &self.right_witness
    }
}

impl Encodable for AndDeciderInput {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Bytes(self.left.to_abi()),
            Token::Bytes(self.left_witness.to_vec()),
            Token::Bytes(self.right.to_abi()),
            Token::Bytes(self.right_witness.to_vec()),
        ]
    }
}

impl Decodable for AndDeciderInput {
    type Ok = AndDeciderInput;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let left = tuple[0].clone().to_bytes();
        let left_witness = tuple[1].clone().to_bytes();
        let right = tuple[2].clone().to_bytes();
        let right_witness = tuple[3].clone().to_bytes();
        if let (Some(left), Some(left_witness), Some(right), Some(right_witness)) =
            (left, left_witness, right, right_witness)
        {
            Ok(AndDeciderInput::new(
                Property::from_abi(&left).unwrap(),
                Bytes::from(left_witness),
                Property::from_abi(&right).unwrap(),
                Bytes::from(right_witness),
            ))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded = ethabi::decode(
            &[
                ParamType::Bytes,
                ParamType::Bytes,
                ParamType::Bytes,
                ParamType::Bytes,
            ],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

pub struct AndDeciderWitness {}

impl Encodable for AndDeciderWitness {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![]
    }
}

impl Decodable for AndDeciderWitness {
    type Ok = AndDeciderWitness;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        Ok(AndDeciderWitness {})
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded = ethabi::decode(&[], data).map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

pub struct AndDecider {
    decider_manager: DeciderManager,
}

impl Default for AndDecider {
    fn default() -> Self {
        AndDecider {
            decider_manager: DeciderManager {},
        }
    }
}

impl Decider for AndDecider {
    type Input = AndDeciderInput;
    type Witness = AndDeciderWitness;

    fn decide(&self, input: &AndDeciderInput, witness: AndDeciderWitness) -> Decision {
        let left_decider = self
            .decider_manager
            .get_decider(input.get_left().get_decider_id());
        let right_decider = self
            .decider_manager
            .get_decider(input.get_right().get_decider_id());
        let left_decision = left_decider.decide(
            &left_decider.decode_input(input.get_left().get_input()),
            left_decider.decode_witness(input.get_left_witness()),
        );
        let right_decision = right_decider.decide(
            &right_decider.decode_input(input.get_right().get_input()),
            right_decider.decode_witness(input.get_right_witness()),
        );
        if let DecisionStatus::Decided(false) = left_decision.get_outcome() {
            return left_decision;
        }
        return right_decision;
        Decision::new(DecisionStatus::Decided(true), vec![])
    }

    fn check_decision(&self, input: &AndDeciderInput) -> Decision {
        self.decide(input, AndDeciderWitness {})
    }

    fn decode_input(&self, input: &Bytes) -> AndDeciderInput {
        AndDeciderInput::from_abi(&input.to_vec()).unwrap()
    }

    fn decode_witness(&self, input: &Bytes) -> AndDeciderWitness {
        AndDeciderWitness::from_abi(&input.to_vec()).unwrap()
    }
}
