use crate::db::RangeAtBlockDb;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, ImplicationProofElement, PropertyInput};
use crate::DeciderManager;
use abi_utils::abi::Encodable;
use bytes::Bytes;
use merkle_interval_tree::{DoubleLayerTree, DoubleLayerTreeLeaf};
use plasma_db::traits::kvs::KeyValueStore;

/// IncludedAtBlock is decider which decide inclusion of data in merkle interval tree
pub struct IncludedAtBlockDecider {}

impl Default for IncludedAtBlockDecider {
    fn default() -> Self {
        Self {}
    }
}

impl Decider for IncludedAtBlockDecider {
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        inputs: &[PropertyInput],
    ) -> Result<Decision, Error> {
        let block_number = decider.get_variable(&inputs[0]).to_integer();
        let state_update = decider.get_variable(&inputs[1]).to_state_update();
        let db: RangeAtBlockDb<T> = RangeAtBlockDb::new(decider.get_range_db());
        let range_at_block_record = db.get_witness(
            block_number,
            state_update.get_deposit_contract_address(),
            state_update.get_range(),
        )?;
        let leaf = DoubleLayerTreeLeaf {
            data: Bytes::from(state_update.to_abi()),
            end: state_update.get_range().get_end(),
            address: state_update.get_deposit_contract_address(),
        };
        let inclusion_bounds_result = DoubleLayerTree::verify(
            &leaf,
            range_at_block_record.inclusion_proof.clone(),
            &range_at_block_record.root,
        );
        if !inclusion_bounds_result {
            return Err(Error::from(ErrorKind::CannotDecide));
        }
        Ok(Decision::new(
            true,
            vec![ImplicationProofElement::new(
                DeciderManager::included_at_block_decider(inputs.to_vec()),
                Some(range_at_block_record.inclusion_proof.clone()),
            )],
        ))
    }
}
