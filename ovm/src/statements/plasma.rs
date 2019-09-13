use crate::types::{
    BlockRangeQuantifierInput, ForAllSuchThatInput, Integer, IsDeprecatedDeciderInput, Property,
    PropertyFactory, Quantifier, QuantifierResultItem, StateUpdate,
};
use plasma_core::data_structure::Range;

/// Creates plasma checkpoint property
/// for all b such that b < block_number:
///   for all p such that included in block(b):
///      Or(b, Included(p), Excluded(b, p))
pub fn create_plasma_property(specified_block_number: Integer, range: Range) -> Property {
    Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
        Quantifier::NonnegativeIntegerLessThanQuantifier(specified_block_number),
        Some(PropertyFactory::new(Box::new(move |item| {
            if let QuantifierResultItem::Integer(block_number) = item {
                create_coin_range_property(block_number, range)
            } else {
                panic!("invalid type in PropertyFactory");
            }
        }))),
    )))
}

pub fn create_coin_range_property(block_number: Integer, range: Range) -> Property {
    Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
        Quantifier::BlockRangeQuantifier(BlockRangeQuantifierInput::new(block_number, range)),
        Some(PropertyFactory::new(Box::new(move |item| {
            // TODO: fix
            // IsDeprecatedDecider(IsdeprecatedDeciderInput(state_update))
            // IsDeprecatedDecider = input.state_update.property.decide()
            if let QuantifierResultItem::PlasmaDataBlock(plasma_data_block) = item {
                println!("create_coin_range_property {:?}", block_number);
                Property::IsDeprecatedDecider(Box::new(IsDeprecatedDeciderInput::new(
                    StateUpdate::from(plasma_data_block),
                )))
            } else {
                panic!("invalid type in PropertyFactory");
            }
        }))),
    )))
}

#[cfg(test)]
mod tests {

    use super::create_plasma_property;
    use crate::db::{RangeAtBlockDb, TransactionDb};
    use crate::deciders::signed_by_decider::Verifier as SignatureVerifier;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{Integer, OwnershipDeciderInput, PlasmaDataBlock, Property, StateUpdate};
    use bytes::Bytes;
    use ethereum_types::{Address, H256};
    use ethsign::SecretKey;
    use merkle_interval_tree::{MerkleIntervalNode, MerkleIntervalTree};
    use plasma_core::data_structure::abi::Encodable;
    use plasma_core::data_structure::{Range, Transaction, TransactionParams};
    use plasma_db::impls::kvs::CoreDbMemoryImpl;
    use plasma_db::traits::kvs::KeyValueStore;

    fn store_inclusion_witness<KVS: KeyValueStore>(decider: &PropertyExecutor<KVS>) {
        let db = RangeAtBlockDb::new(decider.get_range_db());
        let tx_db = TransactionDb::new(decider.get_range_db());
        for i in 0..10 {
            let block_number = Integer(i);
            store_an_inclusion_witness(&db, &tx_db, block_number, i % 2 == 0);
        }
    }

    fn store_an_inclusion_witness<KVS: KeyValueStore>(
        db: &RangeAtBlockDb<KVS>,
        tx_db: &TransactionDb<KVS>,
        block_number: Integer,
        inclusion: bool,
    ) {
        let raw_key =
            hex::decode("c87509a1c067bbde78beb793e6fa76530b6382a4c0241e5e4a9ec0a0f44dc0d3")
                .unwrap();
        let secret_key = SecretKey::from_raw(&raw_key).unwrap();
        let alice: Address = secret_key.public().address().into();
        let property =
            Property::OwnershipDecider(OwnershipDeciderInput::new(StateUpdate::default()));
        let mut leaves = vec![];
        let mut first_state_update_opt: Option<StateUpdate> = None;
        for i in 0..100 {
            let state_update = StateUpdate::new(
                block_number,
                Range::new(i * 30, i * 30 + 100),
                property.get_decider_id(),
                Bytes::from(alice.as_bytes()),
            );
            if i == 0 {
                first_state_update_opt = Some(state_update.clone());
            }
            leaves.push(MerkleIntervalNode::Leaf {
                end: i * 30 + 100,
                data: if inclusion {
                    Bytes::from(state_update.to_abi())
                } else {
                    Bytes::from(H256::zero().as_bytes())
                },
            })
        }
        let tree = MerkleIntervalTree::generate(&leaves);
        let root = tree.get_root();
        let inclusion_proof = tree.get_inclusion_proof(0, 100);
        if let MerkleIntervalNode::Leaf { data, .. } = &leaves[0] {
            let plasma_data_block: PlasmaDataBlock = PlasmaDataBlock::new(
                Integer(0),
                Range::new(0, 100),
                root.clone(),
                inclusion,
                property.get_decider_id(),
                block_number,
                data.clone(),
            );
            assert!(db
                .store_witness(inclusion_proof, plasma_data_block.clone())
                .is_ok());
        }
        let tx_body =
            TransactionParams::new(Address::zero(), Range::new(0, 100), Bytes::default()).to_abi();
        let signature = SignatureVerifier::sign(&secret_key, &Bytes::from(tx_body));
        let first_state_update = first_state_update_opt.unwrap();
        tx_db.put_transaction(
            first_state_update.get_block_number().0,
            Transaction::new(
                Address::zero(),
                Range::new(0, 100),
                Bytes::default(),
                signature,
            ),
        );
    }

    /// plasma
    #[test]
    fn test_succeed_to_decide_plasma_checkpoint() {
        let block_number = Integer(10);
        let range = Range::new(0, 100);
        let checkpoint_property = create_plasma_property(block_number, range);
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        store_inclusion_witness(&decider);
        let result = decider.decide(&checkpoint_property);
        assert!(result.is_ok());
    }
}
