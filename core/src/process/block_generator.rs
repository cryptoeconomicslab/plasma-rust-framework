extern crate ethereum_types;

use crate::data_structure::block::Block;
use crate::data_structure::error::Error;
use crate::data_structure::StateUpdate;
use ethereum_types::H256;

pub struct BlockGenerator {}

impl BlockGenerator {
    pub fn generate(state_updates: &[StateUpdate]) -> Result<Block, Error> {
        // TODO: caluculate merkle root
        // copy all transactions
        Ok(Block::new(state_updates, H256::zero()))
    }
}
