use super::utils::string_to_address;
use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::{Decodable, Encodable, Integer};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use ovm::types::{core::Property, StateUpdate};
use plasma_core::data_structure::Range;
use plasma_db::prelude::*;
const MIN_RANGE: u64 = 0;
const MAX_RANGE: u64 = std::u64::MAX;

#[derive(Clone, Debug, PartialEq, Eq, AbiEncodable, AbiDecodable)]
struct StateUpdateRecord {
    block_number: Integer,
    deposit_contract_address: Address,
    property: Property,
}

impl StateUpdateRecord {
    pub fn new(
        block_number: Integer,
        deposit_contract_address: Address,
        property: Property,
    ) -> Self {
        Self {
            block_number,
            deposit_contract_address,
            property,
        }
    }

    pub fn into_state_update(self, start: u64, end: u64) -> StateUpdate {
        StateUpdate::new(
            self.block_number,
            self.deposit_contract_address,
            Range::new(start, end),
            self.property,
        )
    }
}

impl From<&StateUpdate> for StateUpdateRecord {
    fn from(state_update: &StateUpdate) -> StateUpdateRecord {
        StateUpdateRecord::new(
            state_update.get_block_number(),
            state_update.get_deposit_contract_address(),
            state_update.get_property().clone(),
        )
    }
}

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
            .map(|range| {
                StateUpdateRecord::from_abi(range.get_value())
                    .unwrap()
                    .into_state_update(range.get_start(), range.get_end())
            })
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
            .put(start, end, &StateUpdateRecord::from(state_update).to_abi())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi_utils::Integer;
    use ovm::property_executor::DeciderManager;
    use ovm::types::StateUpdate;
    use plasma_core::data_structure::Range;

    #[test]
    fn test_put() {
        let db = CoreDbMemoryImpl::open("test");
        let range_db = RangeDbImpl::from(db);
        let mut state_db = StateDb::new(&range_db);

        let state_update1 = StateUpdate::new(
            Integer::new(0),
            Address::zero(),
            Range::new(0, 100),
            DeciderManager::ownership(vec![]),
        );
        let _ = state_db.put_verified_state_update(&state_update1);

        let state_update2 = StateUpdate::new(
            Integer::new(1),
            Address::zero(),
            Range::new(0, 50),
            DeciderManager::ownership(vec![]),
        );
        let _ = state_db.put_verified_state_update(&state_update2);

        let state_updates = state_db
            .get_verified_state_updates(Address::zero(), 0, 2000)
            .unwrap();
        assert_eq!(state_updates.len(), 2);
        assert_eq!(state_updates[0].get_range().get_start(), 0);
        assert_eq!(state_updates[0].get_range().get_end(), 50);
        assert_eq!(state_updates[1].get_range().get_start(), 50);
        assert_eq!(state_updates[1].get_range().get_end(), 100);
    }
}
