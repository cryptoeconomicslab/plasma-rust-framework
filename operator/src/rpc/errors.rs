//
// Created on Wed May 08 2019
//
// Copyright (c) 2019 Cryptoeconomics Lab, Inc.
// This file is part of Plasma Chamber.
//

use jsonrpc_core::Error as JsonRpcError;

/// invalid parameters
pub fn invalid_params<T: std::fmt::Debug>(details: T) -> JsonRpcError {
    JsonRpcError::invalid_params(format!("{:?}", details))
}
