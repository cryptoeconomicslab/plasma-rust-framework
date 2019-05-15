//
// Created on Wed May 08 2019
//
// Copyright (c) 2019 Cryptoeconomics Lab, Inc.
// This file is part of Plasma Chamber.
//

extern crate jsonrpc_core;
extern crate plasma_core;

use super::plasmarpc::PlasmaRpc;
use crate::context::ChainContext;
use jsonrpc_core::{Error as JsonRpcError, ErrorCode, Result};
use plasma_core::data_structure::{Block, SignedTransaction};

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
    fn send_transaction(&self, signed_transaction: SignedTransaction) -> Result<bool> {
        self.chain_context.append(&signed_transaction);
        Ok(true)
    }
    fn generate_block(&self) -> Result<Block> {
        self.chain_context
            .generate()
            .map_err(|_err| JsonRpcError::new(ErrorCode::InternalError))
    }
}

#[cfg(test)]
mod tests {
    use super::PlasmaRpc;
    use super::PlasmaRpcImpl;
    use jsonrpc_http_server::jsonrpc_core::IoHandler;

    #[test]
    fn test_protocol_version() {
        let mut io = IoHandler::new();

        let rpc = PlasmaRpcImpl::new();
        io.extend_with(rpc.to_delegate());

        let request = r#"{"jsonrpc": "2.0", "method": "protocolVersion", "params": [], "id": 1}"#;
        let response = r#"{"jsonrpc":"2.0","result":"0.1.0","id":1}"#;

        assert_eq!(io.handle_request_sync(request), Some(response.to_string()));
    }

}
