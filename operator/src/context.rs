//
// Created on Wed May 08 2019
//
// Copyright (c) 2019 Cryptoeconomics Lab, Inc.
// This file is part of Plasma Chamber.
//

extern crate plasma_core;

use super::error::Error;
use parking_lot::RwLock;
use plasma_core::data_structure::{Block, SignedTransaction};
use plasma_core::process::BlockGenerator;
use std::sync::Arc;

#[derive(Default)]
pub struct ChainContext {
    signed_transactions: Arc<RwLock<Vec<SignedTransaction>>>,
}

impl ChainContext {
    pub fn new() -> Self {
        ChainContext {
            signed_transactions: Arc::new(RwLock::new(vec![])),
        }
    }
    pub fn append(&self, signed_transaction: &SignedTransaction) {
        self.signed_transactions
            .write()
            .push(signed_transaction.clone());
    }
    pub fn generate(&self) -> Result<Block, Error> {
        BlockGenerator::generate(&self.signed_transactions.read().clone()).map_err(Into::into)
    }
}
