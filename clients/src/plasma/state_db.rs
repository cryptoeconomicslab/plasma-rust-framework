use bytes::Bytes;
use ovm::types::StateUpdate;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::Range;
use plasma_db::error::Error;
use plasma_db::range::Range as RangeWithValue;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::traits::rangestore::RangeStore;
use plasma_db::RangeDbImpl;

const MIN_RANGE: u64 = 0;
const MAX_RANGE: u64 = std::u64::MAX;

pub struct StateDb<'a, KVS: KeyValueStore> {
    db: &'a RangeDbImpl<KVS>,
}

impl<'a, KVS: KeyValueStore> StateDb<'a, KVS> {
    pub fn new(range_db: &'a RangeDbImpl<KVS>) -> Self {
        StateDb { db: range_db }
    }

    pub fn get_all_state_updates(&self) -> Result<Vec<StateUpdate>, Error> {
        self.get_verified_state_updates(MIN_RANGE, MAX_RANGE)
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
                let mut first_block = StateUpdate::from_abi(intersection.get_value()).unwrap();
                first_block.set_range(Range::new(range.get_start(), intersection.get_start()));

                let mut third_block = StateUpdate::from_abi(intersection.get_value()).unwrap();
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
