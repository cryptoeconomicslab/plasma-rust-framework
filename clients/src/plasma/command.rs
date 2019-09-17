use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ovm::types::Integer;
use plasma_core::data_structure::abi::{Decodable, Encodable};

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct Command {
    pub command_type: Integer,
    pub body: Bytes,
}

impl Command {
    pub fn new(command_type: Integer, body: Bytes) -> Self {
        Self { command_type, body }
    }
    pub fn create_fetch_block_request(block_number: Integer) -> Self {
        Command {
            command_type: Integer(1),
            body: Bytes::from(FetchBlockRequest { block_number }.to_abi()),
        }
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct FetchBlockRequest {
    pub block_number: Integer,
}

impl FetchBlockRequest {
    pub fn new(block_number: Integer) -> Self {
        Self { block_number }
    }
}
