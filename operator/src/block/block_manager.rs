use super::block_db::BlockDb;
use crate::error::Error;
use plasma_core::data_structure::StateUpdate;
use plasma_core::types::BlockNumber;
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
    pub fn initiate(&self) -> Result<(), Error> {
        self.db.set_block_number(BlockNumber::new(0))
    }
    pub fn get_next_block_number(&self) -> Result<BlockNumber, Error> {
        self.db.get_next_block_number()
    }
    /// Adds new StateUpdate
    pub fn add_pending_state_update(&mut self, state_update: &StateUpdate) -> Result<(), Error> {
        self.db.add_pending_state_update(state_update)
    }
    pub fn get_pending_state_updates(&self) -> Result<Vec<StateUpdate>, Error> {
        self.db.get_pending_state_updates()
    }
    /// Fixes next block, submits Merkle root and increments block number
    pub fn submit_next_block(&self) -> Result<(), Error> {
        let _state_updates = self.get_pending_state_updates()?;
        self.db.finalize_block()?;
        // generate merkle root from _state_updates
        // submit block.get_root()
        Ok(())
    }
}
