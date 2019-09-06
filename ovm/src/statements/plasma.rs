use crate::types::{
    BlockRangeQuantifierInput, ForAllSuchThatInput, IncludedAtBlockInput, InputType, Integer,
    Property, Quantifier,
};
use bytes::Bytes;
use plasma_core::data_structure::Range;

/// Creates plasma checkpoint property
/// for all b such that b < block_number:
///   for all p such that included in block(b):
///      Or(b, Included(p), Excluded(b, p))
pub fn create_plasma_property(specified_block_number: Integer, range: Range) -> Property {
    Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
        Quantifier::NonnegativeIntegerLessThanQuantifier(InputType::ConstantInteger(
            specified_block_number,
        )),
        Bytes::from("block"),
        create_coin_range_property(
            &InputType::placeholder("block"),
            &InputType::ConstantRange(range),
        ),
    )))
}

pub fn create_coin_range_property(block_number: &InputType, range: &InputType) -> Property {
    Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
        Quantifier::BlockRangeQuantifier(BlockRangeQuantifierInput::new(
            block_number.clone(),
            range.clone(),
        )),
        Bytes::from("state"),
        Property::IncludedAtBlockDecider(Box::new(IncludedAtBlockInput::new(
            block_number.clone(),
            InputType::placeholder("state"),
        ))),
    )))
}

#[cfg(test)]
mod tests {

    use super::create_plasma_property;
    use crate::db::RangeAtBlockDb;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{
        InputType, Integer, PlasmaDataBlock, PreimageExistsInput, Property, Witness,
    };
    use bytes::Bytes;
    use ethereum_types::H256;
    use merkle_interval_tree::{MerkleIntervalNode, MerkleIntervalTree};
    use plasma_core::data_structure::abi::Encodable;
    use plasma_core::data_structure::Range;
    use plasma_db::impls::kvs::CoreDbLevelDbImpl;
    use plasma_db::traits::kvs::KeyValueStore;

    fn store_inclusion_witness<KVS: KeyValueStore>(decider: &PropertyExecutor<KVS>) {
        let db = RangeAtBlockDb::new(decider.get_range_db());
        for i in 0..10 {
            let block_number = Integer(i);
            store_an_inclusion_witness(&db, block_number, i % 2 == 0);
        }
    }

    fn store_an_inclusion_witness<KVS: KeyValueStore>(
        db: &RangeAtBlockDb<KVS>,
        block_number: Integer,
        inclusion: bool,
    ) {
        let property = Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(
            InputType::placeholder("hash"),
        )));
        let mut leaves = vec![];
        for i in 0..100 {
            leaves.push(MerkleIntervalNode::Leaf {
                end: i * 30 + 100,
                data: if inclusion {
                    Bytes::from(property.to_abi())
                } else {
                    Bytes::from(H256::zero().as_bytes())
                },
            })
        }
        let tree = MerkleIntervalTree::generate(&leaves);
        let root = tree.get_root();
        let inclusion_proof = tree.get_inclusion_proof(0, 100);
        let plasma_data_block: PlasmaDataBlock = PlasmaDataBlock::new(
            Integer(0),
            Range::new(0, 100),
            root.clone(),
            inclusion,
            property,
        );
        let witness =
            Witness::IncludedInIntervalTreeAtBlock(inclusion_proof, plasma_data_block.clone());
        assert!(db
            .store_witness(block_number, &plasma_data_block, &witness)
            .is_ok());
    }

    /// plasma
    #[test]
    fn test_succeed_to_decide_plasma_checkpoint() {
        let block_number = Integer(10);
        let range = Range::new(0, 100);
        let checkpoint_property = create_plasma_property(block_number, range);
        let mut decider: PropertyExecutor<CoreDbLevelDbImpl> = Default::default();
        store_inclusion_witness(&decider);
        let result = decider.decide(&checkpoint_property);
        assert!(result.is_ok());
    }
}
