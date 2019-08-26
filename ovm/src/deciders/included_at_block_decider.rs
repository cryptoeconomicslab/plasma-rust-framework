use crate::db::RangeAtBlockDb;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{
    Decider, Decision, ImplicationProofElement, IncludedAtBlockInput, Property,
    QuantifierResultItem, Witness,
};
use bytes::Bytes;
use merkle_interval_tree::{MerkleIntervalNode, MerkleIntervalTree};
use plasma_core::data_structure::abi::Encodable;
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
        decider: &mut PropertyExecutor<T>,
        input: &IncludedAtBlockInput,
    ) -> Result<Decision, Error> {
        let db: RangeAtBlockDb<T> = RangeAtBlockDb::new(decider.get_range_db());
        if let (
            QuantifierResultItem::Integer(block_number),
            QuantifierResultItem::PlasmaDataBlock(plasma_data_block),
        ) = (
            decider.replace(input.get_block_number()),
            decider.replace(input.get_plasma_data_block()),
        ) {
            let witness = db.get_witness(block_number, &plasma_data_block)?;
            if let Witness::IncludedInIntervalTreeAtBlock(inclusion_proof, _) = witness.clone() {
                let leaf = MerkleIntervalNode::Leaf {
                    end: plasma_data_block.get_updated_range().get_end(),
                    data: Bytes::from(plasma_data_block.get_property().to_abi()),
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
                return Ok(Decision::new(
                    true,
                    vec![ImplicationProofElement::new(
                        Property::IncludedAtBlockDecider(Box::new(input.clone())),
                        Some(witness.clone()),
                    )],
                ));
            }
        }
        panic!("invalid input")
    }
}
