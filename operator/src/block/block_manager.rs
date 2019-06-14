use super::block_db::BlockDb;
use super::types::BlockNumber;
use crate::error::Error;
use plasma_core::data_structure::StateUpdate;
use plasma_core::process::BlockGenerator;
use plasma_db::range::Range;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;

/// Block Manager
pub struct BlockManager<D> {
    db: BlockDb<D>,
}

impl<D> Default for BlockManager<D>
where
    D: DatabaseTrait + KeyValueStore<Range>,
{
    fn default() -> Self {
        Self {
            db: Default::default(),
        }
    }
}

impl<D> BlockManager<D>
where
    D: DatabaseTrait + KeyValueStore<Range>,
{
    pub fn get_next_block_number(&self) -> Result<BlockNumber, Error> {
        self.db.get_next_block_number()
    }
    /// Adds new StateUpdate
    pub fn add_pending_state_update(&mut self, state_update: StateUpdate) -> Result<(), Error> {
        self.db.add_pending_state_update(state_update)
    }
    pub fn get_pending_state_updates(&self) -> Result<Vec<StateUpdate>, Error> {
        self.db.get_pending_state_updates()
    }
    /// Submits the next block
    pub fn submit_next_block(&self) -> Result<(), Error> {
        let state_updates = self.get_pending_state_updates()?;
        self.db.finalize_block()?;
        let _block = BlockGenerator::generate(&state_updates)?;
        // submit block.get_root()
        Ok(())
    }
}
