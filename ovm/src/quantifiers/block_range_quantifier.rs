use crate::property_executor::PropertyExecutor;
use crate::types::{
    BlockRangeQuantifierInput, PlasmaDataBlock, QuantifierResult, QuantifierResultItem, Witness,
};
use bytes::Bytes;
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
    pub fn get_all_quantified<KVS>(
        decider: &PropertyExecutor<KVS>,
        input: &BlockRangeQuantifierInput,
    ) -> QuantifierResult
    where
        KVS: KeyValueStore,
    {
        let block_number = input.get_block_number();
        let range = input.get_coin_range();
        let result = decider
            .get_range_db()
            .bucket(&Bytes::from("range_at_block"))
            .bucket(&block_number.into())
            .get(range.get_start(), range.get_end())
            .unwrap();
        let sum = result
            .iter()
            .fold(0, |acc, r| acc + r.get_end() - r.get_start());
        let full_range_included: bool = sum == (range.get_end() - range.get_start());
        let plasma_data_blocks: Vec<PlasmaDataBlock> = result
            .iter()
            .map(|r| Witness::from_abi(r.get_value()).unwrap())
            .map(|w| {
                if let Witness::IncludedInIntervalTreeAtBlock(_, plasma_data_block) = w {
                    plasma_data_block.clone()
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
    }
}
