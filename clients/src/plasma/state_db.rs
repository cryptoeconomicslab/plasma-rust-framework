use super::utils::string_to_address;
use abi_utils::{Decodable, Encodable};
use bytes::Bytes;
use ethereum_types::Address;
use ovm::types::StateUpdate;
use plasma_db::prelude::*;

const MIN_RANGE: u64 = 0;
const MAX_RANGE: u64 = std::u64::MAX;

pub struct StateDb<'a, KVS: KeyValueStore> {
    db: &'a RangeDbImpl<KVS>,
}

impl<'a, KVS: KeyValueStore> StateDb<'a, KVS> {
    pub fn new(range_db: &'a RangeDbImpl<KVS>) -> Self {
        StateDb { db: range_db }
    }

    pub fn get_all_state_updates(&self) -> Result<Vec<StateUpdate>, PlasmaDbError> {
        let mut result = vec![];
        // TODO: get dynamically
        let mut state_updates =
            self.get_verified_state_updates(Address::zero(), MIN_RANGE, MAX_RANGE)?;
        result.append(&mut state_updates);
        let mut state_updates = self.get_verified_state_updates(
            string_to_address("0000000000000000000000000000000000000001"),
            MIN_RANGE,
            MAX_RANGE,
        )?;
        result.append(&mut state_updates);
        Ok(result)
    }

    pub fn get_verified_state_updates(
        &self,
        deposit_contract_address: Address,
        start: u64,
        end: u64,
    ) -> Result<Vec<StateUpdate>, PlasmaDbError> {
        let res = self
            .db
            .bucket(&Bytes::from(&b"verified_state_updates"[..]))
            .bucket(&Bytes::from(deposit_contract_address.as_bytes()))
            .get(start, end)?
            .iter()
            .map(|range| StateUpdate::from_abi(range.get_value()).unwrap())
            .collect();
        Ok(res)
    }

    pub fn put_verified_state_update(
        &mut self,
        state_update: &StateUpdate,
    ) -> Result<(), PlasmaDbError> {
        let start = state_update.get_range().get_start();
        let end = state_update.get_range().get_end();
        let deposit_contract_address = state_update.get_deposit_contract_address();

        self.db
            .bucket(&Bytes::from(&b"verified_state_updates"[..]))
            .bucket(&Bytes::from(deposit_contract_address.as_bytes()))
            .put(start, end, &state_update.to_abi())
    }
}
