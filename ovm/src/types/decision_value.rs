use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use plasma_core::data_structure::abi::{Decodable, Encodable};

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct DecisionValue {
    decision: bool,
    witness: Bytes,
}

impl DecisionValue {
    pub fn new(decision: bool, witness: Bytes) -> Self {
        Self { decision, witness }
    }
    pub fn get_decision(&self) -> bool {
        self.decision
    }
    pub fn get_witness(&self) -> &Bytes {
        &self.witness
    }
}
