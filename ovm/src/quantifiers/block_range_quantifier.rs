use crate::property_executor::PropertyExecutor;
use crate::types::{
    BlockRangeQuantifierInput, PlasmaDataBlock, QuantifierResult, QuantifierResultItem, Witness,
};
use bytes::Bytes;
use ethereum_types::H256;
use merkle_interval_tree::{MerkleIntervalNode, MerkleIntervalTree};
use plasma_core::data_structure::abi::Decodable;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::traits::rangestore::RangeStore;

pub struct BlockRangeQuantifier {}

impl Default for BlockRangeQuantifier {
    fn default() -> Self {
        Self {}
    }
}

impl BlockRangeQuantifier {
    pub fn verify_exclusion(plasma_data_block: &PlasmaDataBlock, inclusion_proof: &Bytes) -> bool {
        let leaf = MerkleIntervalNode::Leaf {
            end: plasma_data_block.get_updated_range().get_end(),
            data: Bytes::from(H256::zero().as_bytes()),
        };
        let inclusion_bounds_result = MerkleIntervalTree::verify(
            &leaf,
            plasma_data_block.get_index(),
            inclusion_proof.clone(),
            plasma_data_block.get_root(),
        );
        inclusion_bounds_result.is_ok()
    }
    pub fn get_all_quantified<KVS>(
        decider: &PropertyExecutor<KVS>,
        input: &BlockRangeQuantifierInput,
    ) -> QuantifierResult
    where
        KVS: KeyValueStore,
    {
        if let (QuantifierResultItem::Integer(block_number), QuantifierResultItem::Range(range)) = (
            decider.replace(&input.block_number),
            decider.replace(&input.coin_range),
        ) {
            let result = decider
                .get_range_db()
                .bucket(&Bytes::from("range_at_block"))
                .bucket(&(block_number).into())
                .get(range.get_start(), range.get_end())
                .unwrap();
            let sum = result
                .iter()
                .filter_map(|r| r.get_intersection(range.get_start(), range.get_end()))
                .fold(0, |acc, r| acc + r.get_end() - r.get_start());
            let mut full_range_included: bool = sum == (range.get_end() - range.get_start());
            let plasma_data_blocks: Vec<PlasmaDataBlock> = result
                .iter()
                .map(|r| Witness::from_abi(r.get_value()).unwrap())
                .filter_map(move |w| {
                    if let Witness::IncludedInIntervalTreeAtBlock(
                        inclusion_proof,
                        plasma_data_block,
                    ) = w
                    {
                        if plasma_data_block.get_is_included() {
                            Some(plasma_data_block.clone())
                        } else {
                            if !Self::verify_exclusion(&plasma_data_block, &inclusion_proof) {
                                full_range_included = false
                            }
                            None
                        }
                    } else {
                        panic!("invalid witness")
                    }
                })
                .collect();
            QuantifierResult::new(
                plasma_data_blocks
                    .iter()
                    .map(|p| QuantifierResultItem::PlasmaDataBlock(p.clone()))
                    .collect(),
                full_range_included,
            )
        } else {
            panic!("invalid input")
        }
    }
}
