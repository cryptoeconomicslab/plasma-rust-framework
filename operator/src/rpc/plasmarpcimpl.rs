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
use jsonrpc_core::Result;
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

impl From<ChainContext> for PlasmaRpcImpl {
    fn from(context: ChainContext) -> Self {
        PlasmaRpcImpl {
            chain_context: context,
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
        Ok(self.chain_context.append(&transaction).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::ChainContext;
    use super::PlasmaRpc;
    use super::PlasmaRpcImpl;
    use bytes::Bytes;
    use ethereum_types::{Address, H256};
    use jsonrpc_http_server::jsonrpc_core::IoHandler;
    use plasma_core::data_structure::{StateObject, StateUpdate, Transaction, Witness};
    use predicate_plugins::parameters::PredicateParameters;
    use predicate_plugins::OwnershipPredicateParameters;

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

        let context = ChainContext::new();
        assert!(context.initiate().is_ok());
        let deposit_state = StateUpdate::new(
            StateObject::new(Address::zero(), Bytes::from(&b"data"[..])),
            0,
            200,
            10,
            Address::zero(),
        );
        assert!(context.force_deposit(&deposit_state));
        let rpc = PlasmaRpcImpl::from(context);
        io.extend_with(rpc.to_delegate());

        let parameters = OwnershipPredicateParameters::new(
            StateObject::new(Address::zero(), Bytes::from(Address::zero().as_bytes())),
            15,
            20,
        );
        let transaction = Transaction::new(
            Address::zero(),
            0,
            100,
            Transaction::create_method_id(&b"send(address)"[..]),
            parameters.encode(),
            &Witness::new(H256::zero(), H256::zero(), 0),
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
