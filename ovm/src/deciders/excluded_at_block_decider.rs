use crate::db::RangeAtBlockDb;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{
    Decider, Decision, ImplicationProofElement, IncludedAtBlockInput, Property, Witness,
};
use bytes::Bytes;
use ethereum_types::H256;
use merkle_interval_tree::{MerkleIntervalNode, MerkleIntervalTree};
use plasma_db::traits::kvs::KeyValueStore;

/// IncludedAtBlock is decider which decide inclusion of data in merkle interval tree
pub struct ExcludedAtBlockDecider {}

impl Default for ExcludedAtBlockDecider {
    fn default() -> Self {
        Self {}
    }
}

impl Decider for ExcludedAtBlockDecider {
    type Input = IncludedAtBlockInput;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &IncludedAtBlockInput,
    ) -> Result<Decision, Error> {
        let db: RangeAtBlockDb<T> = RangeAtBlockDb::new(decider.get_range_db());
        let witness = db.get_witness(input)?;
        if let Witness::IncludedInIntervalTreeAtBlock(inclusion_proof, _) = witness.clone() {
            let plasma_data_block = input.get_plasma_data_block();
            let leaf = MerkleIntervalNode::Leaf {
                end: plasma_data_block.get_updated_range().get_end(),
                data: Bytes::from(H256::zero().as_bytes()),
            };
            let inclusion_bounds_result = MerkleIntervalTree::verify(
                &leaf,
                plasma_data_block.get_index(),
                inclusion_proof,
                plasma_data_block.get_root(),
            );
            if inclusion_bounds_result.is_err() {
                return Err(Error::from(ErrorKind::CannotDecide));
            }
            Ok(Decision::new(
                true,
                vec![ImplicationProofElement::new(
                    Property::ExcludedAtBlockDecider(Box::new(input.clone())),
                    Some(witness.clone()),
                )],
            ))
        } else {
            panic!("invalid witness")
        }
    }
}
