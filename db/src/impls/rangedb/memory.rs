use crate::error::{Error, ErrorKind};
use crate::range::Range;
use crate::traits::db::DatabaseTrait;
use crate::traits::rangestore::RangeStore;
use parking_lot::RwLock;
use std::collections::BTreeMap;

pub struct RangeDbMemoryImpl {
    ranges: RwLock<BTreeMap<u64, Range>>,
}

impl DatabaseTrait for RangeDbMemoryImpl {
    fn open(_dbname: &str) -> Self {
        Self {
            ranges: RwLock::new(BTreeMap::new()),
        }
    }
    fn close(&self) {}
}

impl RangeDbMemoryImpl {
    fn validate_range(start: u64, end: u64) -> bool {
        start < end
    }
    pub fn del_batch(&self, start: u64, end: u64) -> Result<Vec<Range>, Error> {
        let ranges = self.get(start, end)?;
        let mut db = self.ranges.write();
        for range in ranges.clone().into_iter() {
            db.remove(&range.get_end());
        }
        Ok(ranges.clone())
    }
    pub fn put_batch(&mut self, ranges: &[Range]) -> Result<(), Error> {
        let mut db = self.ranges.write();
        for range in ranges.into_iter() {
            db.insert(range.get_end(), range.clone());
        }
        Ok(())
    }
}

impl RangeStore for RangeDbMemoryImpl {
    fn get(&self, start: u64, end: u64) -> Result<Vec<Range>, Error> {
        let db = self.ranges.read();
        let mut result = vec![];
        println!("get() {:?}, {:?}", start, end);
        for (key, range) in db.iter() {
            println!("get {:?}, {:?}", key, range);
            if start < *key {
                if !range.intersect(start, end) {
                    break;
                } else {
                    result.push(range.clone());
                }
            }
        }
        Ok(result)
    }
    fn del(&self, start: u64, end: u64) -> Result<Vec<Range>, Error> {
        self.del_batch(start, end)
    }
    fn put(&mut self, start: u64, end: u64, value: &[u8]) -> Result<(), Error> {
        let input_ranges = self.del_batch(start, end)?;
        let mut output_ranges = vec![];
        if !Self::validate_range(start, end) {
            return Err(Error::from(ErrorKind::Dammy));
        }
        if !input_ranges.is_empty() && input_ranges[0].get_start() < start {
            output_ranges.push(Range::new(
                input_ranges[0].get_start(),
                start,
                &input_ranges[0].get_value(),
            ));
        }
        if !input_ranges.is_empty() {
            let last_range = &input_ranges[input_ranges.len() - 1];
            if end < last_range.get_end() {
                output_ranges.push(Range::new(
                    end,
                    last_range.get_end(),
                    &last_range.get_value(),
                ));
            }
        }
        output_ranges.push(Range::new(start, end, value));
        if self.put_batch(&output_ranges).is_ok() {
            Ok(())
        } else {
            Err(Error::from(ErrorKind::Dammy))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RangeDbMemoryImpl;
    use crate::traits::db::DatabaseTrait;
    use crate::traits::rangestore::RangeStore;

    #[test]
    fn test_get_same_range() {
        let mut db = RangeDbMemoryImpl::open("test");
        assert_eq!(db.put(0, 100, b"Alice is owner").is_ok(), true);
        assert_eq!(db.put(100, 200, b"Bob is owner").is_ok(), true);
        let result1 = db.get(100, 200).unwrap();
        assert_eq!(result1.is_empty(), false);
        assert_eq!(result1[0].get_start(), 100);
        assert_eq!(result1[0].get_value(), b"Bob is owner");
    }

    #[test]
    fn test_get_small_range() {
        let mut db = RangeDbMemoryImpl::open("test");
        assert_eq!(db.put(0, 100, b"Alice is owner").is_ok(), true);
        assert_eq!(db.put(100, 120, b"Bob is owner").is_ok(), true);
        assert_eq!(db.put(120, 180, b"Carol is owner").is_ok(), true);
        let result1 = db.get(20, 50).unwrap();
        assert_eq!(result1.is_empty(), false);
        assert_eq!(result1[0].get_start(), 0);
        assert_eq!(result1[0].get_value(), b"Alice is owner");
        assert_eq!(result1.len(), 1);
    }

    #[test]
    fn test_get_large_range() {
        let mut db = RangeDbMemoryImpl::open("test");
        assert_eq!(db.put(0, 100, b"Alice is owner").is_ok(), true);
        assert_eq!(db.put(100, 120, b"Bob is owner").is_ok(), true);
        assert_eq!(db.put(120, 180, b"Carol is owner").is_ok(), true);
        let result1 = db.get(20, 150).unwrap();
        assert_eq!(result1.is_empty(), false);
        assert_eq!(result1[0].get_start(), 0);
        assert_eq!(result1[0].get_value(), b"Alice is owner");
        assert_eq!(result1.len(), 3);
    }
}
