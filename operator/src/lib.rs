//
// Created on Wed May 08 2019
//
// Copyright (c) 2019 Cryptoeconomics Lab, Inc.
// This file is part of Plasma Chamber.
//

extern crate failure;
extern crate serde;
extern crate serde_derive;

/// error definitions.
pub mod error;
/// APIs for JSON RPC.
pub mod rpc;
/// Plasma JSON RPC server.
pub mod server;

use self::server::get_server;
use env_logger;
use std::env;

/// entry point of plasma chain.
pub fn entry() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    println!("Hello, operator!!");
    let server = get_server(&Default::default()).expect("Unable to start rpc server.");
    server.wait();
}
