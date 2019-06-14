use super::block_db::BlockDb;
use super::types::BlockNumber;
use crate::error::Error;
use plasma_core::data_structure::StateUpdate;
use plasma_core::process::BlockGenerator;

pub struct BlockManager {
    db: BlockDb,
}

impl Default for BlockManager {
    fn default() -> Self {
        Self {
            db: Default::default(),
        }
    }
}

impl BlockManager {
    pub fn get_next_block_number(&self) -> Result<BlockNumber, Error> {
        self.db.get_next_block_number()
    }
    pub fn add_pending_state_update(&mut self, state_update: StateUpdate) {
        self.db.add_pending_state_update(state_update);
    }
    pub fn get_pending_state_updates(&self) -> &Vec<StateUpdate> {
        &self.pending_state_updates
    }
    pub fn submit_next_block(&self) -> Result<(), Error> {
        let _block = BlockGenerator::generate(&self.get_state_updates())?;
        // submit block.get_root()
        Ok(())
    }
}
