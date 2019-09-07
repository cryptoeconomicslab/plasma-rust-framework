use crate::db::TransactionDb;
use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, OwnershipDeciderInput, Property};
use crate::DecideMixin;
use plasma_core::data_structure::{StateUpdate, Transaction};
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
        let witness =
            db.get_transactions(state_update.get_block_number().0, state_update.get_range())?;

        // TODO: verify signature.
        if true {
            Ok(Decision::new(true, vec![]))
        } else {
            panic!("invalid witness");
        }
    }
}
