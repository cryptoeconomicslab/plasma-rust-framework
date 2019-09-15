use crate::db::RangeAtBlockDb;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, ImplicationProofElement, PropertyInput};
use crate::DeciderManager;
use merkle_interval_tree::{MerkleIntervalNode, MerkleIntervalTree};
use plasma_db::traits::kvs::KeyValueStore;

/// IncludedAtBlock is decider which decide inclusion of data in merkle interval tree
pub struct IncludedAtBlockDecider {}

impl Default for IncludedAtBlockDecider {
    fn default() -> Self {
        Self {}
    }
}

impl Decider for IncludedAtBlockDecider {
    fn decide<T: KeyValueStore>(
        decider: &mut PropertyExecutor<T>,
        inputs: &[PropertyInput],
    ) -> Result<Decision, Error> {
        let block_number = decider.get_variable(&inputs[0]).to_integer();
        let plasma_data_block = decider.get_variable(&inputs[0]).to_plasma_data_block();
        let db: RangeAtBlockDb<T> = RangeAtBlockDb::new(decider.get_range_db());
        let range_at_block_record =
            db.get_witness(block_number, plasma_data_block.get_updated_range())?;
        let leaf = MerkleIntervalNode::Leaf {
            end: plasma_data_block.get_updated_range().get_end(),
            data: plasma_data_block.get_data().clone(),
        };
        let inclusion_bounds_result = MerkleIntervalTree::verify(
            &leaf,
            plasma_data_block.get_index(),
            range_at_block_record.inclusion_proof.clone(),
            plasma_data_block.get_root(),
        );
        if inclusion_bounds_result.is_err() {
            return Err(Error::from(ErrorKind::CannotDecide));
        }
        Ok(Decision::new(
            true,
            vec![ImplicationProofElement::new(
                DeciderManager::included_at_block_decider(inputs.to_vec()),
                Some(range_at_block_record.inclusion_proof.clone()),
            )],
        ))
    }
}
