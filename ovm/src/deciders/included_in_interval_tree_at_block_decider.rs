use crate::db::RangeAtBlockDb;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{
    Decider, Decision, ImplicationProofElement, IncludedInIntervalTreeAtBlockInput, Property,
    Witness,
};
use bytes::Bytes;
use merkle_interval_tree::{MerkleIntervalNode, MerkleIntervalTree};
use plasma_core::data_structure::abi::Encodable;
use plasma_core::data_structure::Range;
use plasma_db::traits::kvs::KeyValueStore;

/// IncludedInIntervalTreeAtBlock is decider which decide inclusion of data in merkle interval tree
pub struct IncludedInIntervalTreeAtBlock {}

impl Default for IncludedInIntervalTreeAtBlock {
    fn default() -> Self {
        Self {}
    }
}

impl Decider for IncludedInIntervalTreeAtBlock {
    type Input = IncludedInIntervalTreeAtBlockInput;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &IncludedInIntervalTreeAtBlockInput,
    ) -> Result<Decision, Error> {
        let db: RangeAtBlockDb<T> = RangeAtBlockDb::new(decider.get_range_db());
        let witness = db.get_witness(input)?;
        if let Witness::IncludedInIntervalTreeAtBlock(inclusion_proof, data_block) = witness.clone()
        {
            let leaf = MerkleIntervalNode::Leaf {
                end: input.get_coin_range().get_start(),
                data: Bytes::from(data_block.get_property().to_abi()),
            };
            let root: Bytes = input.get_block_number().into();
            let inclusion_bounds_result =
                MerkleIntervalTree::verify(&leaf, 5, inclusion_proof, &root);
            if inclusion_bounds_result.is_err() {
                return Err(Error::from(ErrorKind::CannotDecide));
            }
            let implicit_bounds = inclusion_bounds_result.unwrap();
            let inclusion_bounds: Range =
                Range::new(implicit_bounds.get_start(), implicit_bounds.get_end());
            // Insert inclusion decision
            let _relevant_inclusion: Range = input
                .get_coin_range()
                .get_overlapping_range(&data_block.get_updated_range());
            if input.get_coin_range().get_end() == data_block.get_updated_range().get_end() {
                return Err(Error::from(ErrorKind::CannotDecide));
            }
            // Insert exclusion decision
            let _relevant_exclusion = Range::new(
                data_block.get_updated_range().get_end(),
                inclusion_bounds.get_end(),
            );
            Ok(Decision::new(
                true,
                vec![ImplicationProofElement::new(
                    Property::IncludedInIntervalTreeAtBlockDecider(input.clone()),
                    Some(witness.clone()),
                )],
            ))
        } else {
            panic!("invalid witness")
        }
    }
}
