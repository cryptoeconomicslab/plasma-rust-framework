use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{
    Decider, Decision, DecisionValue, ImplicationProofElement, IncludedInIntervalTreeAtBlockInput,
    Property, Witness,
};
use bytes::Bytes;
use merkle_interval_tree::{MerkleIntervalNode, MerkleIntervalTree};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::Range;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::traits::rangestore::RangeStore;

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
        witness: Option<Witness>,
    ) -> Result<Decision, Error> {
        if let Some(Witness::IncludedInIntervalTreeAtBlock(inclusion_proof, data_block)) =
            witness.clone()
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
            let relevant_inclusion: Range = input
                .get_coin_range()
                .get_overlapping_range(&data_block.get_updated_range());
            let inclusion_decision_value = DecisionValue::new(true, witness.clone().unwrap());
            decider
                .get_range_db()
                .bucket(&Bytes::from("range_at_block"))
                .bucket(&input.get_block_number().into())
                .put(
                    relevant_inclusion.get_start(),
                    relevant_inclusion.get_end(),
                    &inclusion_decision_value.to_abi(),
                )
                .map_err::<Error, _>(Into::into)?;
            if input.get_coin_range().get_end() == data_block.get_updated_range().get_end() {
                return Err(Error::from(ErrorKind::CannotDecide));
            }
            // Insert exclusion decision
            let relevant_exclusion = Range::new(
                data_block.get_updated_range().get_end(),
                inclusion_bounds.get_end(),
            );
            let exclusion_decision_value = DecisionValue::new(true, witness.clone().unwrap());
            decider
                .get_range_db()
                .bucket(&Bytes::from("range_at_block"))
                .bucket(&input.get_block_number().into())
                .put(
                    relevant_exclusion.get_start(),
                    relevant_exclusion.get_end(),
                    &exclusion_decision_value.to_abi(),
                )
                .map_err::<Error, _>(Into::into)?;
            Ok(Decision::new(true, vec![]))
        } else {
            panic!("invalid witness")
        }
    }
    fn check_decision<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &IncludedInIntervalTreeAtBlockInput,
    ) -> Result<Decision, Error> {
        let decision_key = input.get_coin_range();
        let result = decider
            .get_range_db()
            .bucket(&Bytes::from("range_at_block"))
            .bucket(&input.get_block_number().into())
            .get(decision_key.get_start(), decision_key.get_end())
            .map_err::<Error, _>(Into::into)?;
        let decision_value =
            DecisionValue::from_abi(result[0].get_value()).map_err::<Error, _>(Into::into)?;
        Ok(Decision::new(
            decision_value.get_decision(),
            vec![ImplicationProofElement::new(
                Property::IncludedInIntervalTreeAtBlockDecider(input.clone()),
                Some(decision_value.get_witness().clone()),
            )],
        ))
    }
}
