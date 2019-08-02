use super::Decision;
use crate::data_structure::abi::{Decodable, Encodable};

pub trait Decider {
    type Input: Decodable + Encodable;
    type Witness: Decodable + Encodable;
    fn decide(&self, input: &Self::Input, witness: Self::Witness) -> Decision;
    fn check_decision(&self, input: &Self::Input) -> Decision;
    fn decode_input(&self, input: &bytes::Bytes) -> Self::Input;
    fn decode_witness(&self, input: &bytes::Bytes) -> Self::Witness;
}
