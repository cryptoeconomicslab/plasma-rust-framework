pub mod atomic_state;
pub mod channel;
pub mod swap;

use crate::types::{Integer, Property, PropertyInput};
use crate::DeciderManager;
pub use atomic_state::*;
use bytes::Bytes;
pub use channel::*;
use ethereum_types::Address;
use plasma_core::data_structure::Range;
pub use swap::*;

/// Creates plasma checkpoint property
/// for all b such that b < block_number:
///   for all p such that included in block(b):
///      Or(b, Included(p), Excluded(b, p))
pub fn create_plasma_property(
    specified_block_number: Integer,
    deposit_contract_address: Address,
    range: Range,
) -> Property {
    DeciderManager::for_all_such_that_decider(
        DeciderManager::q_less_than(vec![PropertyInput::ConstantInteger(specified_block_number)]),
        Bytes::from("block"),
        DeciderManager::for_all_such_that_decider(
            DeciderManager::q_block(vec![
                PropertyInput::Placeholder(Bytes::from("block")),
                PropertyInput::ConstantAddress(deposit_contract_address),
                PropertyInput::ConstantRange(range),
            ]),
            Bytes::from("state_update"),
            DeciderManager::is_deprecated(vec![PropertyInput::Placeholder(Bytes::from(
                "state_update",
            ))]),
        ),
    )
}

#[cfg(test)]
mod tests {

    use super::create_plasma_property;
    use crate::db::{RangeAtBlockDb, TransactionDb};
    use crate::deciders::signed_by_decider::Verifier as SignatureVerifier;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{Integer, PropertyInput, StateUpdate};
    use crate::DeciderManager;
    use abi_utils::Encodable;
    use bytes::Bytes;
    use ethereum_types::{Address, H256};
    use ethsign::SecretKey;
    use merkle_interval_tree::{DoubleLayerTree, DoubleLayerTreeLeaf};
    use plasma_core::data_structure::{Metadata, Range, Transaction, TransactionParams};
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
        let property = DeciderManager::ownership(vec![
            PropertyInput::Placeholder(Bytes::from("state_update")),
            PropertyInput::ConstantAddress(alice),
        ]);
        let deposit_contract_address: Address = Address::zero();
        let mut leaves = vec![];
        let mut first_state_update_opt: Option<StateUpdate> = None;
        for i in 0..100 {
            let state_update = StateUpdate::new(
                block_number,
                deposit_contract_address,
                Range::new(i * 30, i * 30 + 100),
                property.clone(),
            );
            if i == 0 {
                first_state_update_opt = Some(state_update.clone());
            }
            leaves.push(DoubleLayerTreeLeaf {
                address: deposit_contract_address,
                end: i * 30 + 100,
                data: if inclusion {
                    Bytes::from(state_update.to_abi())
                } else {
                    Bytes::from(H256::zero().as_bytes())
                },
            })
        }
        let tree = DoubleLayerTree::generate(&leaves);
        let root = tree.get_root();
        let inclusion_proof = tree.get_inclusion_proof(deposit_contract_address, 0);
        let first_state_update = first_state_update_opt.unwrap();
        assert!(db
            .store_witness(root, inclusion, inclusion_proof, first_state_update.clone())
            .is_ok());

        let tx_body =
            TransactionParams::new(Address::zero(), Range::new(0, 100), Bytes::default()).to_abi();
        let signature = SignatureVerifier::sign(&secret_key, &Bytes::from(tx_body));
        tx_db.put_transaction(
            first_state_update.get_block_number().0,
            Transaction::new(
                Address::zero(),
                Range::new(0, 100),
                Bytes::default(),
                signature,
                Metadata::default(),
            ),
        );
    }

    /// plasma
    #[test]
    fn test_succeed_to_decide_plasma_checkpoint() {
        let block_number = Integer(10);
        let deposit_contract_address: Address = Address::zero();
        let range = Range::new(0, 100);
        let checkpoint_property =
            create_plasma_property(block_number, deposit_contract_address, range);
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        store_inclusion_witness(&decider);
        let result = decider.decide(&checkpoint_property);
        assert!(result.is_ok());
    }
}
