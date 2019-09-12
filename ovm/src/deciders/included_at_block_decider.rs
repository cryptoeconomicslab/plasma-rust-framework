use crate::db::RangeAtBlockDb;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, ImplicationProofElement, IncludedAtBlockInput, Property};
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
    type Input = IncludedAtBlockInput;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &IncludedAtBlockInput,
    ) -> Result<Decision, Error> {
        let db: RangeAtBlockDb<T> = RangeAtBlockDb::new(decider.get_range_db());
        let range_at_block_record = db.get_witness(input)?;
        let plasma_data_block = input.get_plasma_data_block();
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
                Property::IncludedAtBlockDecider(Box::new(input.clone())),
                Some(range_at_block_record.inclusion_proof.clone()),
            )],
        ))
    }
}
