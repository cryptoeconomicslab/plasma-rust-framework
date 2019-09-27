use super::atomic_state::create_atomic_state;
use crate::property_executor::PropertyExecutor;
use crate::types::{Property, PropertyInput, StateUpdate};
use crate::DeciderManager;
use bytes::Bytes;
use ethereum_types::Address;
use plasma_db::traits::kvs::KeyValueStore;

pub fn create_channel_state_object_for_variables<KVS: KeyValueStore>(
    decider: &PropertyExecutor<KVS>,
    inputs: &[PropertyInput],
) -> Property {
    let my_address = decider.get_variable(&inputs[0]).to_address();
    let counter_party_address = decider.get_variable(&inputs[1]).to_address();
    let corresponding_state_update = decider.get_variable(&inputs[2]).to_state_update();
    create_channel_state_object(
        my_address,
        counter_party_address,
        corresponding_state_update,
    )
}

/// channel property for Plasma
pub fn create_channel_state_object(
    my_address: Address,
    counter_party_address: Address,
    corresponding_state_update: StateUpdate,
) -> Property {
    /*
     * There exists tx such that state_update.is_same_coin_range(tx):
     *   There exists corresponding_property = create_channel(counter_party_address, my_address):
     *     There exists correspondent such that correspondent = create_state_update(corresponding_range, corresponding_property):
     *       Or(
     *         And(IncludedAt(correspondent), SignedBy(tx, to_address), SignedBy(tx, counter_party_address)),
     *         And(Not(IncludedAt(correspondent)), SignedBy(tx, counter_party_address))
     *       )
     *
     * This can be compiled to small property.
     *
     * There exists tx such that state_update.is_same_coin_range(tx):
     *   AtomicStateUpdate(
     *     create_channel(counter_party_address, my_address),
     *     corresponding_range,
     *     And(SignedBy(tx, to_address), SignedBy(tx, counter_party_address)),
     *     SignedBy(tx, counter_party_address)
     *   )
     *
     */
    DeciderManager::there_exists_such_that(vec![
        PropertyInput::ConstantProperty(DeciderManager::q_tx(vec![PropertyInput::Placeholder(
            Bytes::from("state_update"),
        )])),
        PropertyInput::ConstantBytes(Bytes::from("tx")),
        PropertyInput::ConstantProperty(create_atomic_state(
            corresponding_state_update.get_block_number(),
            corresponding_state_update.get_deposit_contract_address(),
            corresponding_state_update.get_range(),
            vec![
                PropertyInput::ConstantAddress(counter_party_address),
                PropertyInput::ConstantAddress(my_address),
                PropertyInput::Placeholder(Bytes::from("state_update")),
            ],
            DeciderManager::and_decider(
                DeciderManager::signed_by_decider(vec![
                    PropertyInput::ConstantAddress(my_address),
                    PropertyInput::Placeholder(Bytes::from("tx")),
                ]),
                DeciderManager::signed_by_decider(vec![
                    PropertyInput::ConstantAddress(counter_party_address),
                    PropertyInput::Placeholder(Bytes::from("tx")),
                ]),
            ),
            DeciderManager::signed_by_decider(vec![
                PropertyInput::ConstantAddress(my_address),
                PropertyInput::Placeholder(Bytes::from("tx")),
            ]),
        )),
    ])
}

#[cfg(test)]
mod tests {

    use super::create_channel_state_object;
    use crate::db::{RangeAtBlockDb, SignedByDb, TransactionDb};
    use crate::deciders::signed_by_decider::Verifier as SignatureVerifier;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{PlasmaDataBlock, Property, QuantifierResultItem, StateUpdate};
    use crate::DeciderManager;
    use abi_utils::abi::Encodable;
    use abi_utils::Integer;
    use bytes::Bytes;
    use ethereum_types::{Address, H256};
    use ethsign::SecretKey;
    use merkle_interval_tree::{DoubleLayerTree, DoubleLayerTreeLeaf};
    use plasma_core::data_structure::{Metadata, Range, Transaction, TransactionParams};
    use plasma_db::impls::kvs::CoreDbMemoryImpl;

    fn make_state_update(
        block_number: Integer,
        deposit_contract_address: Address,
        range: Range,
        corresponding_deposit_contract_address: Address,
        corresponding_range: Range,
        alice: Address,
        bob: Address,
    ) -> (Property, StateUpdate) {
        let dammy_property = DeciderManager::preimage_exists_decider(vec![]);
        let corresponding_state_update = StateUpdate::new(
            block_number,
            corresponding_deposit_contract_address,
            corresponding_range,
            dammy_property,
        );
        let property = create_channel_state_object(alice, bob, corresponding_state_update.clone());
        (
            property.clone(),
            StateUpdate::new(
                block_number,
                deposit_contract_address,
                range,
                property.clone(),
            ),
        )
    }

    #[test]
    fn test_succeed_to_decide_channel() {
        let raw_key_alice =
            hex::decode("c87509a1c067bbde78beb793e6fa76530b6382a4c0241e5e4a9ec0a0f44dc0d3")
                .unwrap();
        let raw_key_bob =
            hex::decode("ae6ae8e5ccbfb04590405997ee2d52d2b330726137b875053c36d94e974d162f")
                .unwrap();
        let secret_key = SecretKey::from_raw(&raw_key_alice).unwrap();
        let secret_key_bob = SecretKey::from_raw(&raw_key_bob).unwrap();
        let alice: Address = secret_key.public().address().into();
        let bob = secret_key_bob.public().address().into();
        let block_number = Integer(10);
        let deposit_contract_address = Address::random();
        let range = Range::new(0, 100);
        let corresponding_deposit_contract_address = Address::random();
        let corresponding_range = Range::new(100, 200);

        let (property, state_update) = make_state_update(
            block_number,
            deposit_contract_address,
            range,
            corresponding_deposit_contract_address,
            corresponding_range,
            alice,
            bob,
        );
        let (_, corresponding_state_update) = make_state_update(
            block_number,
            corresponding_deposit_contract_address,
            corresponding_range,
            deposit_contract_address,
            range,
            bob,
            alice,
        );
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        let tx_db = TransactionDb::new(decider.get_range_db());
        let signed_by_db = SignedByDb::new(decider.get_db());
        let range_at_block_db = RangeAtBlockDb::new(decider.get_range_db());
        let tx_params =
            TransactionParams::new(Address::zero(), Range::new(0, 100), Bytes::default());
        let tx_body = Bytes::from(tx_params.to_abi());
        let signature = SignatureVerifier::sign(&secret_key, &tx_body);
        let signature_bob = SignatureVerifier::sign(&secret_key_bob, &tx_body);
        tx_db.put_transaction(
            state_update.get_block_number().0,
            Transaction::from_params(tx_params, signature.clone(), Metadata::default()),
        );
        assert!(signed_by_db
            .store_witness(alice, tx_body.clone(), signature.clone())
            .is_ok());
        assert!(signed_by_db
            .store_witness(bob, tx_body.clone(), signature_bob.clone())
            .is_ok());

        let leaf1 = DoubleLayerTreeLeaf {
            address: deposit_contract_address,
            end: 100,
            data: Bytes::from(state_update.to_abi()),
        };
        let leaf2 = DoubleLayerTreeLeaf {
            address: corresponding_deposit_contract_address,
            end: 100,
            data: Bytes::from(H256::zero().as_bytes()),
        };
        let leaf3 = DoubleLayerTreeLeaf {
            address: corresponding_deposit_contract_address,
            end: 200,
            data: Bytes::from(corresponding_state_update.to_abi()),
        };
        let tree = DoubleLayerTree::generate(&[leaf1, leaf2, leaf3.clone()]);
        let root = tree.get_root();
        let index = tree.get_index(corresponding_deposit_contract_address, &leaf3.data);
        let inclusion_proof =
            tree.get_inclusion_proof(corresponding_deposit_contract_address, index);
        let inclusion_bounds_result =
            DoubleLayerTree::verify(&leaf3, inclusion_proof.clone(), &root);
        assert!(inclusion_bounds_result);
        let plasma_data_block = PlasmaDataBlock::new(
            corresponding_deposit_contract_address,
            Range::new(100, 200),
            root.clone(),
            true,
            block_number,
            leaf3.data.clone(),
        );
        assert!(range_at_block_db
            .store_witness(root, inclusion_proof, plasma_data_block.clone())
            .is_ok());

        decider.set_variable(
            Bytes::from("state_update"),
            QuantifierResultItem::StateUpdate(state_update),
        );
        let result = decider.decide(&property);
        assert!(result.is_ok());
        assert!(result.ok().unwrap().get_outcome());
    }
}
