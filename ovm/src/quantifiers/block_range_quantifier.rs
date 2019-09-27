use crate::db::RangeAtBlockRecord;
use crate::property_executor::PropertyExecutor;
use crate::types::{PropertyInput, QuantifierResult, QuantifierResultItem, StateUpdate};
use abi_utils::Decodable;
use bytes::Bytes;
use ethereum_types::H256;
use merkle_interval_tree::{DoubleLayerTree, DoubleLayerTreeLeaf};
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::traits::rangestore::RangeStore;

pub struct BlockRangeQuantifier {}

impl Default for BlockRangeQuantifier {
    fn default() -> Self {
        Self {}
    }
}

impl BlockRangeQuantifier {
    pub fn verify_exclusion(
        state_update: &StateUpdate,
        inclusion_proof: &Bytes,
        root: &Bytes,
    ) -> bool {
        let leaf = DoubleLayerTreeLeaf {
            address: state_update.get_deposit_contract_address(),
            end: state_update.get_range().get_end(),
            data: Bytes::from(H256::zero().as_bytes()),
        };
        DoubleLayerTree::verify(&leaf, inclusion_proof.clone(), root)
    }
    pub fn get_all_quantified<KVS>(
        decider: &PropertyExecutor<KVS>,
        inputs: &[PropertyInput],
    ) -> QuantifierResult
    where
        KVS: KeyValueStore,
    {
        let block_number = decider.get_variable(&inputs[0]).to_integer();
        let deposit_contract_address = decider.get_variable(&inputs[1]).to_address();
        let range = decider.get_variable(&inputs[2]).to_range();
        let result = decider
            .get_range_db()
            .bucket(&Bytes::from("range_at_block"))
            .bucket(&block_number.into())
            .bucket(&Bytes::from(deposit_contract_address.as_bytes()))
            .get(range.get_start(), range.get_end())
            .unwrap();
        let sum = result
            .iter()
            .filter_map(|r| r.get_intersection(range.get_start(), range.get_end()))
            .fold(0, |acc, r| acc + r.get_end() - r.get_start());
        let mut full_range_included: bool = sum == (range.get_end() - range.get_start());
        let state_updates: Vec<StateUpdate> = result
            .iter()
            .map(|r| RangeAtBlockRecord::from_abi(r.get_value()).unwrap())
            .filter_map(move |record| {
                if record.is_included {
                    Some(record.state_update.clone())
                } else {
                    if !Self::verify_exclusion(
                        &record.state_update,
                        &record.inclusion_proof,
                        &record.root,
                    ) {
                        full_range_included = false
                    }
                    None
                }
            })
            .collect();
        QuantifierResult::new(
            state_updates
                .iter()
                .map(|su| QuantifierResultItem::StateUpdate(su.clone()))
                .collect(),
            full_range_included,
        )
    }
}
