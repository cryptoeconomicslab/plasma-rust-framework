//
// Created on Wed May 15 2019
//
// Copyright (c) 2019 Cryptoeconomics Lab, Inc.
// This file is part of Plasma Chamber.
//

extern crate plasma_core;

use parking_lot::RwLock;
use plasma_core::data_structure::Transaction;
use std::sync::Arc;

#[derive(Default)]
pub struct ChainContext {
    transactions: Arc<RwLock<Vec<Transaction>>>,
}

impl ChainContext {
    pub fn new() -> Self {
        ChainContext {
            transactions: Arc::new(RwLock::new(vec![])),
        }
    }
    pub fn append(&self, signed_transaction: &Transaction) {
        self.transactions.write().push(signed_transaction.clone());
    }
}
