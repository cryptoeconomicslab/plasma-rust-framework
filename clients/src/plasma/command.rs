use super::plasma_block::PlasmaBlock;
use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::{Encodable, Integer};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ovm::types::StateUpdateList;
use plasma_core::data_structure::Transaction;

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
    pub fn create_state_update_list(state_update_list: StateUpdateList) -> Self {
        Command {
            command_type: Integer(2),
            body: Bytes::from(state_update_list.to_abi()),
        }
    }
    pub fn create_plasma_block(plasma_block: PlasmaBlock) -> Self {
        Command {
            command_type: Integer(3),
            body: Bytes::from(plasma_block.to_abi()),
        }
    }
    pub fn create_new_tx_event(new_tx_event: NewTransactionEvent) -> Self {
        Command {
            command_type: Integer(4),
            body: Bytes::from(new_tx_event.to_abi()),
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

/// prev_state_block_number is the block numbers which the transaction deprecated
#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct NewTransactionEvent {
    pub prev_state_block_number: Integer,
    pub transaction: Transaction,
}

impl NewTransactionEvent {
    pub fn new(prev_state_block_number: Integer, transaction: Transaction) -> Self {
        Self {
            prev_state_block_number,
            transaction,
        }
    }
}
