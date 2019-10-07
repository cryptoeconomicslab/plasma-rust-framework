extern crate ethabi;

use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::Integer;
use ethabi::{ParamType, Token};
use ethereum_types::Address;

pub const PAYMENT_TYPE: Integer = Integer(0);
pub const EXCHANGE_TYPE: Integer = Integer(1);

#[derive(Clone, Debug, PartialEq, Eq, AbiDecodable, AbiEncodable)]
pub struct Metadata {
    meta_type: Integer,
    from: Address,
    to: Address,
}

impl Metadata {
    pub fn new(meta_type: Integer, from: Address, to: Address) -> Self {
        Self {
            meta_type,
            from,
            to,
        }
    }
    pub fn get_meta_type(&self) -> Integer {
        self.meta_type
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
            meta_type: PAYMENT_TYPE,
            from: Address::zero(),
            to: Address::zero(),
        }
    }
}
