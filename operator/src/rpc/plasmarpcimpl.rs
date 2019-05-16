//
// Created on Wed May 08 2019
//
// Copyright (c) 2019 Cryptoeconomics Lab, Inc.
// This file is part of Plasma Chamber.
//

extern crate jsonrpc_core;
extern crate plasma_core;
extern crate rlp;

use super::plasmarpc::PlasmaRpc;
use crate::context::ChainContext;
use jsonrpc_core::{Error as JsonRpcError, ErrorCode, Result};
use plasma_core::data_structure::Transaction;

/// Plasma JSON RPC implementation.
#[derive(Default)]
pub struct PlasmaRpcImpl {
    chain_context: ChainContext,
}

impl PlasmaRpcImpl {
    pub fn new() -> PlasmaRpcImpl {
        PlasmaRpcImpl {
            chain_context: Default::default(),
        }
    }
}

impl PlasmaRpc for PlasmaRpcImpl {
    fn protocol_version(&self) -> Result<String> {
        Ok("0.1.0".into())
    }
    fn send_transaction(&self, message: String) -> Result<bool> {
        let rlp_bytes =
            hex::decode(message).map_err(|_err| JsonRpcError::new(ErrorCode::ParseError))?;
        let transaction: Transaction =
            rlp::decode(&rlp_bytes).map_err(|_err| JsonRpcError::new(ErrorCode::ParseError))?;
        self.chain_context.append(&transaction);
        Ok(true)
    }
    fn generate_block(&self) -> Result<String> {
        self.chain_context
            .generate()
            .map(|block| rlp::encode(&block))
            .map(hex::encode)
            .map_err(|_err| JsonRpcError::new(ErrorCode::InternalError))
    }
}

#[cfg(test)]
mod tests {
    use super::PlasmaRpc;
    use super::PlasmaRpcImpl;
    use bytes::Bytes;
    use ethereum_types::Address;
    use jsonrpc_http_server::jsonrpc_core::IoHandler;
    use plasma_core::data_structure::{StateObject, StateUpdate, Transaction};

    #[test]
    fn test_protocol_version() {
        let mut io = IoHandler::new();

        let rpc = PlasmaRpcImpl::new();
        io.extend_with(rpc.to_delegate());

        let request = r#"{"jsonrpc": "2.0", "method": "protocolVersion", "params": [], "id": 1}"#;
        let response = r#"{"jsonrpc":"2.0","result":"0.1.0","id":1}"#;

        assert_eq!(io.handle_request_sync(request), Some(response.to_string()));
    }

    #[test]
    fn test_send_transaction() {
        let mut io = IoHandler::new();

        let rpc = PlasmaRpcImpl::new();
        io.extend_with(rpc.to_delegate());

        let state_object = StateObject::new(Address::zero(), &Bytes::from(&b"parameters"[..]));
        let state_update = StateUpdate::new(0, 0, 0, Address::zero(), state_object);

        let transaction = Transaction::new(state_update, &Bytes::from(&b"witness"[..]));
        let encoded = rlp::encode(&transaction);

        let request = format!(
            r#"{{
                "jsonrpc": "2.0",
                "method": "sendTransaction",
                "params": ["{}"],
                "id": 1
            }}"#,
            hex::encode(encoded),
        );
        let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

        assert_eq!(io.handle_request_sync(&request), Some(response.to_string()));
    }

    #[test]
    fn test_faile_to_send_transaction() {
        let mut io = IoHandler::new();

        let rpc = PlasmaRpcImpl::new();
        io.extend_with(rpc.to_delegate());

        let request = r#"{
            "jsonrpc": "2.0",
            "method": "sendTransaction",
            "params": [""],
            "id": 1
        }"#;
        let response =
            r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":1}"#;
        assert_eq!(io.handle_request_sync(&request), Some(response.to_string()));
    }

}
