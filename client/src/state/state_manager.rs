use crate::error::{Error, ErrorKind};
use crate::state::{StateDb, VerifiedStateUpdate};
use ethereum_types::Address;
use plasma_core::data_structure::{StateQuery, StateQueryResult, StateUpdate, Transaction};
use plasma_db::range::Range;
use plasma_db::traits::{DatabaseTrait, KeyValueStore};
use predicate_plugins::PredicateManager;

pub struct ResultOfExecuteTransaction {
    state_update: Box<StateUpdate>,
    ranges: Box<[VerifiedStateUpdate]>,
}

impl ResultOfExecuteTransaction {
    pub fn new(state_update: StateUpdate, ranges: &[VerifiedStateUpdate]) -> Self {
        ResultOfExecuteTransaction {
            state_update: Box::new(state_update),
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

pub struct StateManager<KVS: KeyValueStore<Range>> {
    db: Box<StateDb<KVS>>,
}

impl<KVS> Default for StateManager<KVS>
where
    KVS: DatabaseTrait + KeyValueStore<Range>,
{
    fn default() -> Self {
        Self {
            db: Default::default(),
        }
    }
}

impl<KVS> StateManager<KVS>
where
    KVS: DatabaseTrait + KeyValueStore<Range>,
{
    /// force to put state update
    pub fn deposit(&self, start: u64, end: u64, state_update: StateUpdate) -> Result<(), Error> {
        self.db
            .put_verified_state_update(&VerifiedStateUpdate::new(start, end, 0, state_update))
    }

    /// Applies state query result to StateManager
    pub fn apply_state_query_result(
        &self,
        state_query_result: &StateQueryResult,
    ) -> Result<(), Error> {
        let state_update = state_query_result.get_state_update();
        self.db.put_verified_state_update(&VerifiedStateUpdate::new(
            state_update.get_range().get_start(),
            state_update.get_range().get_end(),
            state_update.get_block_number(),
            state_update.clone(),
        ))
    }

    /// Execute a transaction
    pub fn execute_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<ResultOfExecuteTransaction, Error> {
        let verified_state_updates = self.db.get_verified_state_updates(
            transaction.get_range().get_start(),
            transaction.get_range().get_end(),
        )?;
        let new_state_updates: Vec<StateUpdate> = verified_state_updates
            .iter()
            .map(|verified_state_update| {
                let predicate_address: Address = verified_state_update
                    .get_state_update()
                    .get_state_object()
                    .get_predicate();
                PredicateManager::get_plugin(predicate_address)
                    .execute_state_transition(verified_state_update.get_state_update(), transaction)
            })
            .collect();
        // new_state_updates should has same state_update
        if new_state_updates.is_empty() {
            return Err(Error::from(ErrorKind::InvalidTransaction));
        }
        let new_state_update: StateUpdate = new_state_updates[0].clone();
        self.db
            .put_verified_state_update(&VerifiedStateUpdate::from(
                new_state_update.get_block_number(),
                &new_state_update,
            ))?;
        Ok(ResultOfExecuteTransaction::new(
            new_state_update,
            &verified_state_updates,
        ))
    }

    pub fn query_state(&self, query: &StateQuery) -> Result<Box<[StateQueryResult]>, Error> {
        let verified_state_updates = self.db.get_verified_state_updates(
            query.get_start().unwrap_or(0),
            query.get_end().unwrap_or(0),
        )?;
        let state_query_result: Vec<StateQueryResult> = verified_state_updates
            .iter()
            .map(|verified_state_update| {
                let result = PredicateManager::get_plugin(query.get_predicate_address())
                    .query_state(verified_state_update.get_state_update(), query.get_params());
                StateQueryResult::new(verified_state_update.get_state_update().clone(), &result)
            })
            .collect();
        Ok(state_query_result.into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::StateManager;
    use super::StateQuery;
    use bytes::Bytes;
    use ethereum_types::{Address, H256};
    use plasma_core::data_structure::{Range, StateObject, StateUpdate, Transaction, Witness};
    use plasma_db::impls::kvs::memory::CoreDbMemoryImpl;
    use predicate_plugins::{OwnershipPredicateParameters, PredicateParameters};

    fn create_state_update(start: u64, end: u64, block_number: u64) -> StateUpdate {
        StateUpdate::new(
            StateObject::new(Address::zero(), Bytes::from(&b"data"[..])),
            Range::new(start, end),
            block_number,
            Address::zero(),
        )
    }

    #[test]
    fn test_execute_transaction() {
        // make state update
        let state_update = create_state_update(0, 100, 1);
        let parameters = OwnershipPredicateParameters::new(
            StateObject::new(Address::zero(), Bytes::from(Address::zero().as_bytes())),
            5,
            10,
        );
        let parameters_bytes = parameters.encode();
        // make transaction
        let transaction = Transaction::new(
            Address::zero(),
            Range::new(0, 100),
            parameters_bytes,
            &Witness::new(H256::zero(), H256::zero(), 0),
        );

        let state_manager: StateManager<CoreDbMemoryImpl> = Default::default();
        let deposit_result = state_manager.deposit(0, 100, state_update);
        assert!(deposit_result.is_ok());
        let result = state_manager.execute_transaction(&transaction);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_transaction_for_partial_range() {
        // make state update
        let state_update = create_state_update(0, 100, 1);
        let parameters = OwnershipPredicateParameters::new(
            StateObject::new(Address::zero(), Bytes::from(Address::zero().as_bytes())),
            5,
            10,
        );
        let parameters_bytes = parameters.encode();
        // make transaction
        let transaction = Transaction::new(
            Address::zero(),
            Range::new(0, 20),
            parameters_bytes,
            &Witness::new(H256::zero(), H256::zero(), 0),
        );

        let state_manager: StateManager<CoreDbMemoryImpl> = Default::default();
        let deposit_result = state_manager.deposit(0, 100, state_update);
        assert!(deposit_result.is_ok());
        let result = state_manager.execute_transaction(&transaction);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_transaction_for_multiple_ranges() {
        // make state update
        let state_update1 = create_state_update(0, 100, 1);
        let state_update2 = create_state_update(100, 200, 2);
        let parameters = OwnershipPredicateParameters::new(
            StateObject::new(Address::zero(), Bytes::from(Address::zero().as_bytes())),
            5,
            10,
        );
        let parameters_bytes = parameters.encode();
        // make transaction
        let transaction = Transaction::new(
            Address::zero(),
            Range::new(50, 150),
            parameters_bytes,
            &Witness::new(H256::zero(), H256::zero(), 0),
        );

        let state_manager: StateManager<CoreDbMemoryImpl> = Default::default();
        assert!(state_manager.deposit(0, 100, state_update1).is_ok());
        assert!(state_manager.deposit(100, 200, state_update2).is_ok());
        let result = state_manager.execute_transaction(&transaction);
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_state() {
        // make state update
        let predicate_address = Address::zero();
        let state_update1 = create_state_update(0, 100, 1);
        let state_update2 = create_state_update(100, 200, 2);
        let state_manager: StateManager<CoreDbMemoryImpl> = Default::default();
        assert!(state_manager.deposit(0, 100, state_update1).is_ok());
        assert!(state_manager.deposit(100, 200, state_update2).is_ok());
        let query = StateQuery::new(
            Address::zero(),
            predicate_address,
            Some(0),
            Some(100),
            Bytes::new(),
        );
        let result = state_manager.query_state(&query);
        assert!(result.is_ok());
        println!("{:?}", result);
        assert_eq!(&result.ok().unwrap()[0].get_result()[0][..], &b"data"[..],);
    }

}
