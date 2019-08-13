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
            if !MerkleIntervalTree::verify(&leaf, 5, inclusion_proof, &root).is_ok() {
                return Err(Error::from(ErrorKind::CannotDecide));
            }
            let relevant_inclusion: Range = input
                .get_coin_range()
                .get_overlapping_range(&data_block.get_updated_range());
            let inclusion_decision_value = DecisionValue::new(true, witness.clone().unwrap());
            decider
                .get_range_db()
                //.bucket(&BaseDbKey::from(&b"range_at_block"[..]))
                //.bucket(&BaseDbKey::from(input.get_block_number()))
                .put(
                    relevant_inclusion.get_start(),
                    relevant_inclusion.get_end(),
                    &inclusion_decision_value.to_abi(),
                )
                .map_err::<Error, _>(Into::into)?;
        } else {
            panic!("invalid witness")
        }

        Ok(Decision::new(true, vec![]))
    }
    fn check_decision<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &IncludedInIntervalTreeAtBlockInput,
    ) -> Result<Decision, Error> {
        let decision_key = input.get_coin_range();
        let result = decider
            .get_range_db()
            //.bucket(&BaseDbKey::from(&b"preimage_exists_decider"[..]))
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
