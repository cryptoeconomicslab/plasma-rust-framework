//
// Created on Wed May 08 2019
//
// Copyright (c) 2019 Cryptoeconomics Lab, Inc.
// This file is part of Plasma Chamber.
//

extern crate jsonrpc_core;
extern crate plasma_core;
extern crate rlp;

use super::errors;
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
        let abi_bytes = hex::decode(message).map_err(errors::invalid_params)?;
        let transaction: Transaction =
            Transaction::from_abi(&abi_bytes).map_err(errors::invalid_params)?;
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
    use ethereum_types::Address;
    use jsonrpc_http_server::jsonrpc_core::IoHandler;
    use plasma_core::data_structure::Transaction;

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

        let parameters_bytes = Vec::from(&b"parameters"[..]);
        let transaction = Transaction::new(
            Address::zero(),
            0,
            0,
            100,
            &Transaction::create_method_id(&b"send(address)"[..]),
            &parameters_bytes,
        );
        let encoded = transaction.to_abi();

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

    /*
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
            r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"RlpExpectedToBeList"},"id":1}"#;
        assert_eq!(io.handle_request_sync(&request), Some(response.to_string()));
    }
    */

}
