use bytes::Bytes;
use ovm::types::StateUpdate;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::Range;
use plasma_db::error::Error;
use plasma_db::range::Range as RangeWithValue;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::traits::rangestore::RangeStore;
use plasma_db::RangeDbImpl;

pub struct StateDb<'a, KVS: KeyValueStore> {
    db: &'a RangeDbImpl<KVS>,
}

impl<'a, KVS: KeyValueStore> StateDb<'a, KVS> {
    pub fn from(range_db: &'a RangeDbImpl<KVS>) -> Self {
        StateDb { db: range_db }
    }

    pub fn get_verified_state_updates(
        &self,
        start: u64,
        end: u64,
    ) -> Result<Vec<StateUpdate>, Error> {
        let res = self
            .db
            .bucket(&Bytes::from(&b"verified_state_updates"[..]))
            .get(start, end)?
            .iter()
            .map(|range| StateUpdate::from_abi(range.get_value()).unwrap())
            .collect();
        Ok(res)
    }

    pub fn put_verified_state_update(&mut self, state_update: StateUpdate) -> Result<(), Error> {
        let start = state_update.get_range().get_start();
        let end = state_update.get_range().get_end();

        let result = self
            .db
            .bucket(&Bytes::from(&b"verified_state_updates"[..]))
            .get(start, end)?;

        if result.len() == 0 {
            // TODO: handle error
            let _ = self
                .db
                .bucket(&Bytes::from(&b"verified_state_updates"[..]))
                .put(start, end, &state_update.to_abi());
            Ok(())
        } else if result.len() == 1 {
            let range = Range::new(result[0].get_start(), result[0].get_end());

            if range.is_subrange(&state_update.get_range()) {
                // split into two or three range if new range is subrange.
                let intersection = result[0].get_intersection(start, end).unwrap();
                let mut first_block =
                    StateUpdate::from_abi(intersection.get_value().clone()).unwrap();
                first_block.set_range(Range::new(range.get_start(), intersection.get_start()));

                let mut third_block =
                    StateUpdate::from_abi(intersection.get_value().clone()).unwrap();
                third_block.set_range(Range::new(intersection.get_end(), range.get_end()));

                let ranges = vec![
                    RangeWithValue::new(
                        range.get_start(),
                        intersection.get_start(),
                        &first_block.to_abi(),
                    ),
                    RangeWithValue::new(
                        intersection.get_start(),
                        intersection.get_end(),
                        &state_update.to_abi(),
                    ),
                    RangeWithValue::new(
                        intersection.get_end(),
                        range.get_end(),
                        &third_block.to_abi(),
                    ),
                ];

                for range in ranges.iter() {
                    if range.validate() {
                        // TODO: handle error
                        let _ = self
                            .db
                            .bucket(&Bytes::from(&b"verified_state_updates"[..]))
                            .put(range.get_start(), range.get_end(), range.get_value());
                    }
                }
            } else if range == state_update.get_range() {
                // override if range is same.
                let _ = self
                    .db
                    .bucket(&Bytes::from(&b"verified_state_updates"[..]))
                    .put(start, end, &state_update.to_abi());
            }
            Ok(())
        } else {
            // TODO: update, split, override
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StateDb;
    use bytes::Bytes;
    use ethereum_types::Address;
    use ovm::types::{Integer, Property, SignedByInput, StateUpdate};
    use plasma_core::data_structure::Range;
    use plasma_db::impls::kvs::memory::CoreDbMemoryImpl;
    use plasma_db::traits::db::DatabaseTrait;
    use plasma_db::RangeDbImpl;

    #[test]
    fn test_store() {
        let base_db = CoreDbMemoryImpl::open("test");
        let db = RangeDbImpl::from(base_db);
        let mut state_db = StateDb::from(&db);
        let address: Address = Address::zero();

        let state_update = StateUpdate::new(
            Integer::new(1),
            Range::new(0, 100),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
            Bytes::from(&b"root"[..]),
        );

        let _ = state_db.put_verified_state_update(state_update);
        let result = state_db.get_verified_state_updates(0, 100).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_split() {
        let base_db = CoreDbMemoryImpl::open("test");
        let db = RangeDbImpl::from(base_db);
        let mut state_db = StateDb::from(&db);
        let address: Address = Address::zero();

        let state_update = StateUpdate::new(
            Integer::new(1),
            Range::new(0, 100),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
            Bytes::from(&b"root"[..]),
        );

        let state_update2 = StateUpdate::new(
            Integer::new(1),
            Range::new(0, 50),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi2"[..]), address)),
            Bytes::from(&b"root"[..]),
        );

        let _ = state_db.put_verified_state_update(state_update);
        let _ = state_db.put_verified_state_update(state_update2);
        let result = state_db.get_verified_state_updates(0, 100).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].get_range(), Range::new(0, 50));
        assert_eq!(result[1].get_range(), Range::new(50, 100));
    }
}
