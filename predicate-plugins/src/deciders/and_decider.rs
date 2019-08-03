use crate::DeciderManager;
use bytes::Bytes;
use ethabi::{ParamType, Token};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{Error, ErrorKind};
use plasma_core::ovm::{Decider, Decision, DecisionStatus, Property};

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
    fn from_tuple(_tuple: &[Token]) -> Result<Self, Error> {
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

impl AndDecider {
    pub fn new(decider_manager: DeciderManager) -> Self {
        AndDecider { decider_manager }
    }
    fn decode_input(input: &Bytes) -> AndDeciderInput {
        AndDeciderInput::from_abi(&input.to_vec()).unwrap()
    }
    fn decode_witness(input: &Bytes) -> AndDeciderWitness {
        AndDeciderWitness::from_abi(&input.to_vec()).unwrap()
    }
}

impl Default for AndDecider {
    fn default() -> Self {
        AndDecider {
            decider_manager: Default::default(),
        }
    }
}

impl Decider for AndDecider {
    fn decide(&self, input_bytes: &Bytes, witness_bytes: &Bytes) -> Decision {
        let input = Self::decode_input(input_bytes);
        let _witness = Self::decode_witness(witness_bytes);
        let left_decider = self
            .decider_manager
            .get_decider(input.get_left().get_decider_id());
        let right_decider = self
            .decider_manager
            .get_decider(input.get_right().get_decider_id());
        let left_decision =
            left_decider.decide(input.get_left().get_input(), input.get_left_witness());
        let right_decision =
            right_decider.decide(input.get_right().get_input(), input.get_right_witness());
        if let DecisionStatus::Decided(false) = left_decision.get_outcome() {
            return left_decision;
        }
        if let DecisionStatus::Decided(false) = right_decision.get_outcome() {
            return right_decision;
        }
        Decision::new(
            DecisionStatus::Decided(true),
            [
                &left_decision.get_implication_proof()[..],
                &right_decision.get_implication_proof()[..],
            ]
            .concat(),
        )
    }

    fn check_decision(&self, input_bytes: &Bytes) -> Decision {
        let witness = AndDeciderWitness {};
        self.decide(input_bytes, &Bytes::from(witness.to_abi()))
    }
}

#[cfg(test)]
mod tests {
    use super::{AndDecider, AndDeciderInput, AndDeciderWitness, Decision, DecisionStatus};
    use crate::deciders::preimage_exists_decider::{
        PreimageExistsInput, PreimageExistsWitness, Verifier,
    };
    use crate::DeciderManager;
    use bytes::Bytes;
    use plasma_core::data_structure::abi::Encodable;
    use plasma_core::ovm::{Decider, Property};

    #[test]
    fn test_decide() {
        let left_input = PreimageExistsInput::new(Verifier::static_hash(&Bytes::from("left")));
        let left_witness = PreimageExistsWitness::new(Bytes::from("left"));
        let right_input = PreimageExistsInput::new(Verifier::static_hash(&Bytes::from("right")));
        let right_witness = PreimageExistsWitness::new(Bytes::from("right"));

        let decider_manager: DeciderManager = Default::default();
        let decider_address = decider_manager.get_preimage_exists_decider_id();
        let and_decider: AndDecider = AndDecider::new(decider_manager);
        let input = AndDeciderInput::new(
            Property::new(decider_address, Bytes::from(left_input.to_abi())),
            Bytes::from(left_witness.to_abi()),
            Property::new(decider_address, Bytes::from(right_input.to_abi())),
            Bytes::from(right_witness.to_abi()),
        );
        let and_decider_witness = AndDeciderWitness {};
        let decided: Decision =
            and_decider.decide(&input.to_abi().into(), &and_decider_witness.to_abi().into());
        assert_eq!(decided.get_outcome(), &DecisionStatus::Decided(true));
        let status = and_decider.check_decision(&input.to_abi().into());
        assert_eq!(status.get_outcome(), &DecisionStatus::Decided(true));
    }

}
