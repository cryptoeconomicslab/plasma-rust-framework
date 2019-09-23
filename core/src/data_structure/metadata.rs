extern crate ethabi;

use abi_derive::{AbiDecodable, AbiEncodable};
use ethabi::{ParamType, Token};
use ethereum_types::Address;

#[derive(Clone, Debug, PartialEq, Eq, AbiDecodable, AbiEncodable)]
pub struct Metadata {
    from: Address,
    to: Address,
}

impl Metadata {
    pub fn new(from: Address, to: Address) -> Self {
        Self { from, to }
    }
    pub fn get_from(&self) -> Address {
        self.from
    }
    pub fn get_to(&self) -> Address {
        self.to
    }
}

impl Default for Metadata {
    fn default() -> Metadata {
        Metadata {
            from: Address::zero(),
            to: Address::zero(),
        }
    }
}
