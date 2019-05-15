//
// Created on Wed May 08 2019
//
// Copyright (c) 2019 Cryptoeconomics Lab, Inc.
// This file is part of Plasma Chamber.
//

/// json rpc server.
use super::error::Error;
use super::rpc::plasmarpc::PlasmaRpc;
use super::rpc::plasmarpcimpl::PlasmaRpcImpl;
use jsonrpc_http_server::jsonrpc_core::IoHandler;
use jsonrpc_http_server::{Server, ServerBuilder};
use std::net::SocketAddr;

/// Options for Plasma JSON RPC server.
pub struct HttpOption {
    threads: usize,
    url: String,
}

impl Default for HttpOption {
    fn default() -> Self {
        Self {
            threads: 3,
            url: "127.0.0.1:8080".to_string(),
        }
    }
}

pub fn get_server(options: &HttpOption) -> Result<Server, Error> {
    let mut io = IoHandler::new();

    let rpc: PlasmaRpcImpl = Default::default();
    io.extend_with(rpc.to_delegate());

    let parsed: Result<SocketAddr, Error> = options.url.parse().map_err(Into::into);
    let url = parsed?;
    ServerBuilder::new(io)
        .threads(options.threads)
        .start_http(&url)
        .map_err(Into::into)
}
