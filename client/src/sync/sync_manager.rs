use super::sync_db::SyncDb;
use crate::error::Error;
use crate::plasma_rpc::HttpPlasmaClient;
use ethereum_types::Address;
use plasma_core::data_structure::{StateQuery, StateQueryResult};
use plasma_core::types::BlockNumber;
use plasma_db::traits::{DatabaseTrait, KeyValueStore};

/// SyncManager synchronize client state with operator's state.
pub struct SyncManager<KVS: KeyValueStore<StateQuery>> {
    sync_db: SyncDb<KVS>,
    uri: String,
}

impl<KVS> SyncManager<KVS>
where
    KVS: KeyValueStore<StateQuery>,
{
    pub fn new(sync_db: SyncDb<KVS>, uri: String) -> Self {
        Self { sync_db, uri }
    }
}

impl<KVS> Default for SyncManager<KVS>
where
    KVS: DatabaseTrait + KeyValueStore<StateQuery>,
{
    fn default() -> Self {
        Self {
            sync_db: SyncDb::new(KVS::open(&"sync")),
            uri: "http://localhost:8080".to_string(),
        }
    }
}

impl<KVS> SyncManager<KVS>
where
    KVS: DatabaseTrait + KeyValueStore<StateQuery>,
{
    /// Callback which is called when new block is submitted
    pub fn sync(&self) -> Vec<StateQueryResult> {
        let state_queries = self.get_all_sync_queries();
        let results: Vec<StateQueryResult> = state_queries
            .iter()
            .filter_map(|s| self.apply_state_query(s))
            .flat_map(|s| s)
            .collect();
        results
    }

    fn get_all_sync_queries(&self) -> Vec<StateQuery> {
        let commitment_contracts = self.sync_db.get_commitment_contracts();
        let deposit_contracts =
            commitment_contracts
                .iter()
                .fold::<Vec<Address>, _>(vec![], |acc, c| {
                    let mut deposit_contracts =
                        self.sync_db.get_deposit_contracts(*c).ok().unwrap();
                    deposit_contracts.extend(acc);
                    deposit_contracts
                });
        deposit_contracts
            .iter()
            .fold::<Vec<StateQuery>, _>(vec![], |acc, c| {
                let mut sync_queries = self.get_sync_queries(*c).ok().unwrap();
                sync_queries.extend(acc);
                sync_queries
            })
    }

    fn apply_state_query(&self, state_query: &StateQuery) -> Option<Vec<StateQueryResult>> {
        // Should reuse?
        let client = HttpPlasmaClient::new(&self.uri).ok().unwrap();
        let state_query_result = client.send_query(&state_query).ok().unwrap();
        Some(state_query_result)
    }

    /// Registers new deposit contract to synchronize
    pub fn add_deposit_contract(
        &self,
        commit_contract: Address,
        deposit_contract: Address,
    ) -> Result<(), Error> {
        self.sync_db
            .add_deposit_contract(commit_contract, deposit_contract)
    }
    /// Removes contract address
    pub fn remove_deposit_contract(
        &self,
        deposit_contract: Address,
        commit_contract: Address,
    ) -> Result<(), Error> {
        self.sync_db
            .remove_deposit_contract(commit_contract, deposit_contract)
    }
    /// Gets last syncronized block number for a deposit_contract
    pub fn get_last_synced_block(
        &self,
        deposit_contract: Address,
    ) -> Result<Option<BlockNumber>, Error> {
        self.sync_db.get_last_synced_block(deposit_contract)
    }
    /// Adds new query for syncronization
    pub fn add_sync_query(
        &self,
        deposit_contract: Address,
        state_query: &StateQuery,
    ) -> Result<(), Error> {
        self.sync_db.add_sync_query(deposit_contract, state_query)
        // self.event_watcher.on(deposit_contract, )
    }
    /// Removes a sync query
    pub fn remove_sync_query(
        &self,
        deposit_contract: Address,
        state_query: &StateQuery,
    ) -> Result<(), Error> {
        self.sync_db
            .remove_sync_query(deposit_contract, state_query)
    }
    /// Gets registered sync queries
    pub fn get_sync_queries(&self, deposit_contract: Address) -> Result<Vec<StateQuery>, Error> {
        self.sync_db.get_sync_queries(deposit_contract)
    }
}

#[cfg(test)]
mod tests {
    use super::SyncManager;
    use bytes::Bytes;
    use ethereum_types::Address;
    use plasma_core::data_structure::StateQuery;
    use plasma_db::impls::kvs::CoreDbMemoryImpl;

    #[test]
    fn test_add_and_remove_deposit_contract() {
        let sync_manager: SyncManager<CoreDbMemoryImpl> = Default::default();
        let deposit_contract: Address = Address::zero();
        let commit_contract: Address = Address::zero();
        assert!(sync_manager
            .add_deposit_contract(deposit_contract, commit_contract)
            .is_ok());
        assert!(sync_manager
            .remove_deposit_contract(deposit_contract, commit_contract)
            .is_ok());
    }

    #[test]
    fn test_add_and_remove_sync_query() {
        let sync_manager: SyncManager<CoreDbMemoryImpl> = Default::default();
        let deposit_contract: Address = Address::zero();
        let predicate_address: Address = Address::zero();
        let query = StateQuery::new(
            Address::zero(),
            predicate_address,
            Some(0),
            Some(100),
            Bytes::new(),
        );

        assert!(sync_manager
            .add_sync_query(deposit_contract, &query)
            .is_ok());
        assert!(sync_manager
            .remove_sync_query(deposit_contract, &query)
            .is_ok());
    }

}
