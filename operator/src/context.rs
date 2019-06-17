//
// Created on Wed May 15 2019
//
// Copyright (c) 2019 Cryptoeconomics Lab, Inc.
// This file is part of Plasma Chamber.
//

extern crate plasma_core;

use crate::block::BlockManager;
use crate::error::Error;
use parking_lot::RwLock;
use plasma_client::state::StateManager;
use plasma_core::data_structure::{StateUpdate, Transaction};
use plasma_db::impls::kvs::CoreDbMemoryImpl;
use std::sync::Arc;

#[derive(Default)]
pub struct ChainContext {
    block_manager: Arc<RwLock<BlockManager<CoreDbMemoryImpl>>>,
    state_manager: Arc<RwLock<StateManager<CoreDbMemoryImpl>>>,
}

impl ChainContext {
    pub fn new() -> Self {
        ChainContext {
            block_manager: Arc::new(RwLock::new(Default::default())),
            state_manager: Arc::new(RwLock::new(Default::default())),
        }
    }
    pub fn initiate(&self) -> Result<(), Error> {
        self.block_manager.write().initiate()
    }
    pub fn force_deposit(&self, state_update: &StateUpdate) -> bool {
        self.state_manager
            .write()
            .deposit(
                state_update.get_start(),
                state_update.get_end(),
                state_update.clone(),
            )
            .is_ok()
    }
    pub fn append(&self, signed_transaction: &Transaction) -> Result<(), Error> {
        let result = self
            .state_manager
            .write()
            .execute_transaction(signed_transaction)
            .map_err::<Error, _>(Into::into)?;
        self.block_manager
            .write()
            .add_pending_state_update(result.get_state_update())
            .map_err::<Error, _>(Into::into)
    }
}
