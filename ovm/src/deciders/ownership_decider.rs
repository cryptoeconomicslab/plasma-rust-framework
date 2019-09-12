use crate::db::TransactionDb;
use crate::deciders::signed_by_decider::Verifier;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, Integer, OwnershipDeciderInput, StateUpdate};
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
    type Input = OwnershipDeciderInput;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &OwnershipDeciderInput,
    ) -> Result<Decision, Error> {
        let db: TransactionDb<T> = TransactionDb::new(decider.get_range_db());
        let state_update = input.get_state_update();
        let txs =
            db.get_transactions(state_update.get_block_number().0, state_update.get_range())?;
        println!("TXS: {:?}", txs);

        if txs.is_empty() {
            return Err(Error::from(ErrorKind::CannotDecide));
        }
        for tx in txs.iter() {
            if Verifier::recover(tx.get_signature(), &Bytes::from(tx.to_body_abi()))
                == input.get_owner_address()
            {
                return Ok(Decision::new(true, vec![]));
            }
        }

        Ok(Decision::new(false, vec![]))
    }
}

impl OwnershipDecider {
    pub fn execute_state_transition(
        prev_state: &StateUpdate,
        transaction: &Transaction,
        next_block_number: Integer,
    ) -> StateUpdate {
        StateUpdate::new(
            next_block_number,
            transaction.get_range(),
            prev_state.get_property_address(),
            transaction.get_parameters().clone(),
        )
    }
}
