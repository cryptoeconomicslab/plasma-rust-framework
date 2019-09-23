use crate::db::TransactionDb;
use crate::deciders::signed_by_decider::Verifier;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, Integer, Property, PropertyInput, StateUpdate};
use abi_utils::Decodable;
use bytes::Bytes;
use plasma_core::data_structure::Transaction;
use plasma_db::traits::kvs::KeyValueStore;

/// OwnershipInput {
///     state_update: StateUpdate,
///     owner: Address,
/// }
///
/// 1. verify there exists transaction with prev_state owner's signature.
/// 2. prev_state's block_number is less than origin_block_number
/// 3. ensure post_state's range is same as transaction's range.
/// 4. transaction.parameters.new_state is same as post_state.state

pub struct OwnershipDecider {}

/// OwnershipDecider property construction
/// Transaction have property which is
/// Input = {
///     state_update: StateUpdate,
///     owner: Address,
/// }
/// property.input have to have owner address.
/// StateUpdate have owner address in property.input
/// Use SignedByDecider's address.

impl Decider for OwnershipDecider {
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        inputs: &[PropertyInput],
    ) -> Result<Decision, Error> {
        let state_update = decider.get_variable(&inputs[0]).to_state_update();
        let owner = decider.get_variable(&inputs[1]).to_address();
        let db: TransactionDb<T> = TransactionDb::new(decider.get_range_db());
        let txs =
            db.get_transactions(state_update.get_block_number().0, state_update.get_range())?;
        if txs.is_empty() {
            return Err(Error::from(ErrorKind::CannotDecide));
        }
        for tx in txs.iter() {
            if Verifier::recover(tx.get_signature(), &Bytes::from(tx.to_body_abi())) == owner {
                return Ok(Decision::new(true, vec![]));
            }
        }

        Ok(Decision::new(false, vec![]))
    }
}

impl OwnershipDecider {
    pub fn execute_state_transition(
        _prev_state: &StateUpdate,
        transaction: &Transaction,
        next_block_number: Integer,
    ) -> StateUpdate {
        StateUpdate::new(
            next_block_number,
            transaction.get_plasma_contract_address(),
            transaction.get_range(),
            Property::from_abi(transaction.get_parameters()).unwrap(),
        )
    }
}
