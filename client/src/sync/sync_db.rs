use crate::error::Error;
use bytes::Bytes;
use ethereum_types::Address;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::StateQuery;
use plasma_core::types::BlockNumber;
use plasma_db::traits::{BaseDbKey, KeyValueStore};

/// Interface for SyncDb
/// SyncDb is used by SyncManager to store
/// see http://spec.plasma.group/en/latest/src/05-client-architecture/sync-db.html
pub trait SyncDbTrait {
    fn get_commitment_contracts(&self) -> Vec<Address>;
    fn get_deposit_contracts(&self, commitment_contract: Address) -> Result<Vec<Address>, Error>;
    fn add_deposit_contract(
        &self,
        commit_contract: Address,
        deposit_contract: Address,
    ) -> Result<(), Error>;
    fn remove_deposit_contract(
        &self,
        commit_contract: Address,
        deposit_contract: Address,
    ) -> Result<(), Error>;
    fn get_last_synced_block(
        &self,
        deposit_contract: Address,
    ) -> Result<Option<BlockNumber>, Error>;
    fn put_last_synced_block(
        &self,
        deposit_contract: Address,
        block: BlockNumber,
    ) -> Result<(), Error>;
    fn add_sync_query(
        &self,
        deposit_contract: Address,
        state_query: &StateQuery,
    ) -> Result<(), Error>;
    fn remove_sync_query(
        &self,
        deposit_contract: Address,
        state_query: &StateQuery,
    ) -> Result<(), Error>;
    fn get_sync_queries(&self, deposit_contract: Address) -> Result<Vec<StateQuery>, Error>;
}

/// Simple implementation for SyncDb
pub struct SyncDb<KVS: KeyValueStore<StateQuery>> {
    db: KVS,
}

impl<KVS> SyncDb<KVS>
where
    KVS: KeyValueStore<StateQuery>,
{
    pub fn new(db: KVS) -> Self {
        Self { db }
    }
}

impl<KVS> SyncDbTrait for SyncDb<KVS>
where
    KVS: KeyValueStore<StateQuery>,
{
    fn get_commitment_contracts(&self) -> Vec<Address> {
        vec![]
    }
    /// Add contract address to synchronize
    fn add_deposit_contract(
        &self,
        commit_contract: Address,
        deposit_contract: Address,
    ) -> Result<(), Error> {
        self.db
            .bucket(&commit_contract.as_bytes().into())
            .put(&deposit_contract.as_fixed_bytes()[..].into(), &b""[..])
            .map_err::<Error, _>(Into::into)
    }
    fn get_deposit_contracts(&self, commit_contract: Address) -> Result<Vec<Address>, Error> {
        Ok(self
            .db
            .bucket(&commit_contract.as_bytes().into())
            .iter_all(&b""[..].into(), Box::new(|_a, _b| true))
            .iter()
            .map(|kv| Address::from_slice(kv.get_key().as_bytes()))
            .collect())
    }
    /// Remove contract address
    fn remove_deposit_contract(
        &self,
        commit_contract: Address,
        deposit_contract: Address,
    ) -> Result<(), Error> {
        self.db
            .bucket(&commit_contract.as_bytes().into())
            .del(&deposit_contract.as_fixed_bytes()[..].into())
            .map_err::<Error, _>(Into::into)
    }
    /// Fetch last synchronized block number
    fn get_last_synced_block(
        &self,
        deposit_contract: Address,
    ) -> Result<Option<BlockNumber>, Error> {
        self.db
            .root()
            .bucket(&BaseDbKey::from(&b"last_synced_block"[..]))
            .get(&deposit_contract.as_bytes().into())
            .map_err::<Error, _>(Into::into)
            .map(|r| r.map(BlockNumber::from))
    }
    fn put_last_synced_block(
        &self,
        deposit_contract: Address,
        block_number: BlockNumber,
    ) -> Result<(), Error> {
        let block_number_bytes: Bytes = block_number.into();
        self.db
            .root()
            .bucket(&BaseDbKey::from(&b"last_synced_block"[..]))
            .put(&deposit_contract.as_bytes().into(), &block_number_bytes)
            .map_err::<Error, _>(Into::into)
    }
    fn add_sync_query(
        &self,
        deposit_contract: Address,
        state_query: &StateQuery,
    ) -> Result<(), Error> {
        let state_query_key: BaseDbKey = state_query.to_hash().into();
        self.db
            .root()
            .bucket(&BaseDbKey::from(&b"sync_queries"[..]))
            .bucket(&deposit_contract.as_bytes().into())
            .put(&state_query_key, &state_query.to_abi())
            .map_err::<Error, _>(Into::into)
    }
    fn remove_sync_query(
        &self,
        deposit_contract: Address,
        state_query: &StateQuery,
    ) -> Result<(), Error> {
        let state_query_key: BaseDbKey = state_query.to_hash().into();
        self.db
            .root()
            .bucket(&BaseDbKey::from(&b"sync_queries"[..]))
            .bucket(&deposit_contract.as_bytes().into())
            .del(&state_query_key)
            .map_err::<Error, _>(Into::into)
    }
    fn get_sync_queries(&self, deposit_contract: Address) -> Result<Vec<StateQuery>, Error> {
        Ok(self
            .db
            .root()
            .bucket(&BaseDbKey::from(&b"sync_queries"[..]))
            .iter_all(&deposit_contract.as_bytes().into(), Box::new(|_a, _b| true))
            .iter()
            .filter_map(|kv| StateQuery::from_abi(kv.get_value()).ok())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::SyncDb;
    use super::SyncDbTrait;
    use bytes::Bytes;
    use ethereum_types::Address;
    use plasma_core::data_structure::StateQuery;
    use plasma_db::impls::kvs::CoreDbMemoryImpl;
    use plasma_db::traits::db::DatabaseTrait;

    #[test]
    fn test_add_and_remove_deposit_contract() {
        let sync_db = SyncDb::new(CoreDbMemoryImpl::open(&"test"));
        let deposit_contract: Address = Address::zero();
        let commit_contract: Address = Address::zero();
        assert!(sync_db
            .add_deposit_contract(commit_contract, deposit_contract)
            .is_ok());
        assert_eq!(
            sync_db
                .get_deposit_contracts(commit_contract)
                .ok()
                .unwrap()
                .len(),
            1
        );
        assert!(sync_db
            .remove_deposit_contract(commit_contract, deposit_contract)
            .is_ok());
        assert!(sync_db
            .get_deposit_contracts(commit_contract)
            .ok()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_add_and_remove_sync_query() {
        let sync_db = SyncDb::new(CoreDbMemoryImpl::open(&"test"));
        let deposit_contract: Address = Address::zero();
        let predicate_address: Address = Address::zero();
        let query = StateQuery::new(
            Address::zero(),
            predicate_address,
            Some(0),
            Some(100),
            Bytes::new(),
        );

        assert!(sync_db.add_sync_query(deposit_contract, &query).is_ok());
        assert_eq!(
            sync_db
                .get_sync_queries(deposit_contract)
                .ok()
                .unwrap()
                .len(),
            1
        );
        assert!(sync_db.remove_sync_query(deposit_contract, &query).is_ok());
        assert!(sync_db
            .get_sync_queries(deposit_contract)
            .ok()
            .unwrap()
            .is_empty());
    }

}
