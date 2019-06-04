use crate::error::Error;
use crate::state::{StateDb, VerifiedStateUpdate};
use ethereum_types::Address;
use plasma_core::data_structure::{StateUpdate, Transaction};
use predicate_plugins::PredicateManager;

pub struct ResultOfExecuteTransaction {
    state_update: Box<StateUpdate>,
    ranges: Box<[VerifiedStateUpdate]>,
}

impl ResultOfExecuteTransaction {
    pub fn new(state_update: &StateUpdate, ranges: &[VerifiedStateUpdate]) -> Self {
        ResultOfExecuteTransaction {
            state_update: Box::new(state_update.clone()),
            ranges: ranges.to_vec().into_boxed_slice(),
        }
    }
    pub fn get_state_update(&self) -> &StateUpdate {
        &self.state_update
    }
    pub fn get_ranges(&self) -> &[VerifiedStateUpdate] {
        &self.ranges
    }
}

pub struct StateManager {
    db: Box<StateDb>,
}

impl Default for StateManager {
    fn default() -> Self {
        Self {
            db: Default::default(),
        }
    }
}

impl StateManager {
    /// force to put state update
    pub fn deposit(&self, start: u64, end: u64, state_update: &StateUpdate) -> Result<(), Error> {
        self.db
            .put_verified_state_update(&VerifiedStateUpdate::new(start, end, 0, state_update))
    }

    /// Execute a transaction
    pub fn execute_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<ResultOfExecuteTransaction, Error> {
        let verified_state_updates = self
            .db
            .get_verified_state_updates(transaction.get_start(), transaction.get_end())?;
        let new_state_updates: Vec<StateUpdate> = verified_state_updates
            .iter()
            .map(|verified_state_update| {
                let predicate_address: &Address = verified_state_update
                    .get_state_update()
                    .get_state_object()
                    .get_predicate();
                PredicateManager::get_plugin(predicate_address)
                    .execute_state_transition(verified_state_update.get_state_update(), transaction)
            })
            .collect();
        // new_state_updates should has same state_update
        let new_state_update: &StateUpdate = &new_state_updates[0];
        self.db
            .put_verified_state_update(&VerifiedStateUpdate::from(
                new_state_update.get_block_number(),
                new_state_update,
            ))?;
        Ok(ResultOfExecuteTransaction::new(
            new_state_update,
            &verified_state_updates,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::StateManager;
    use ethereum_types::{Address, H256};
    use plasma_core::data_structure::{StateObject, StateUpdate, Transaction, Witness};
    use predicate_plugins::OwnershipPredicate;

    fn create_state_update(start: u64, end: u64, block_number: u64) -> StateUpdate {
        StateUpdate::new(
            &StateObject::new(Address::zero(), &b"data"[..]),
            start,
            end,
            block_number,
            Address::zero(),
        )
    }

    #[test]
    fn test_execute_transaction() {
        // make state update
        let state_update = create_state_update(0, 100, 1);
        let parameters =
            OwnershipPredicate::make_parameters(Address::zero(), Address::zero(), 5, 10);
        // make transaction
        let transaction = Transaction::new(
            Address::zero(),
            0,
            100,
            Transaction::create_method_id(&b"send(address)"[..]),
            &parameters,
            &Witness::new(H256::zero(), H256::zero(), 0),
        );

        let state_manager: StateManager = Default::default();
        let deposit_result = state_manager.deposit(0, 100, &state_update);
        assert!(deposit_result.is_ok());
        let result = state_manager.execute_transaction(&transaction);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_transaction_for_partial_range() {
        // make state update
        let state_update = create_state_update(0, 100, 1);
        let parameters =
            OwnershipPredicate::make_parameters(Address::zero(), Address::zero(), 5, 10);
        // make transaction
        let transaction = Transaction::new(
            Address::zero(),
            0,
            20,
            Transaction::create_method_id(&b"send(address)"[..]),
            &parameters,
            &Witness::new(H256::zero(), H256::zero(), 0),
        );

        let state_manager: StateManager = Default::default();
        let deposit_result = state_manager.deposit(0, 100, &state_update);
        assert!(deposit_result.is_ok());
        let result = state_manager.execute_transaction(&transaction);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_transaction_for_multiple_ranges() {
        // make state update
        let state_update1 = create_state_update(0, 100, 1);
        let state_update2 = create_state_update(100, 200, 2);
        let parameters =
            OwnershipPredicate::make_parameters(Address::zero(), Address::zero(), 5, 10);
        // make transaction
        let transaction = Transaction::new(
            Address::zero(),
            50,
            150,
            Transaction::create_method_id(&b"send(address)"[..]),
            &parameters,
            &Witness::new(H256::zero(), H256::zero(), 0),
        );

        let state_manager: StateManager = Default::default();
        assert!(state_manager.deposit(0, 100, &state_update1).is_ok());
        assert!(state_manager.deposit(100, 200, &state_update2).is_ok());
        let result = state_manager.execute_transaction(&transaction);
        assert!(result.is_ok());
    }

}
