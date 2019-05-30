use crate::error::{Error, ErrorKind};
use crate::range::Range;
use crate::traits::db::DatabaseTrait;
use crate::traits::rangestore::RangeStore;
use kvdb::{DBTransaction, KeyValueDB};
use kvdb_memorydb::{create, InMemory};

pub struct RangeDb {
    db: InMemory,
    col: u32,
}

impl DatabaseTrait for RangeDb {
    fn open(_dbname: &str) -> Self {
        RangeDb {
            db: create(8),
            col: 0,
        }
    }
    fn close(&self) {}
}

impl RangeDb {
    fn validate_range(start: u64, end: u64) -> bool {
        start < end
    }
    pub fn del_batch(&self, start: u64, end: u64) -> Result<Vec<Range>, Error> {
        let ranges = self.get(start, end)?;
        let mut tr = DBTransaction::new();
        for range in ranges.clone().into_iter() {
            let query = range.get_end().to_be_bytes();
            tr.delete(Some(self.col), &query);
        }
        self.db.write(tr)?;
        if self.db.flush().is_ok() {
            Ok(ranges)
        } else {
            Err(Error::from(ErrorKind::Dammy))
        }
    }
    pub fn put_batch(&self, ranges: &[Range]) -> Result<(), Error> {
        let mut tr = DBTransaction::new();
        for range in ranges.into_iter() {
            let query = range.get_end().to_be_bytes();
            tr.put(Some(self.col), &query, &rlp::encode(range));
        }
        self.db.write(tr)?;
        if self.db.flush().is_ok() {
            Ok(())
        } else {
            Err(Error::from(ErrorKind::Dammy))
        }
    }
}

impl RangeStore for RangeDb {
    fn get(&self, start: u64, end: u64) -> Result<Vec<Range>, Error> {
        let query = start.to_be_bytes();
        let iter = self.db.iter_from_prefix(Some(self.col), &query);
        let mut result = vec![];
        for (_key, value) in iter {
            let range: Range = rlp::decode(&value).unwrap();
            //result.push(range.clone());
            if start < range.get_end() {
                result.push(range.clone());
                if !range.intersect(start, end) {
                    break;
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
    use super::RangeDb;
    use crate::traits::db::DatabaseTrait;
    use crate::traits::rangestore::RangeStore;

    #[test]
    fn test_put() {
        let mut db = RangeDb::open("test");
        assert_eq!(db.put(0, 100, b"Alice is owner").is_ok(), true);
        assert_eq!(db.put(100, 200, b"Bob is owner").is_ok(), true);
        let result1 = db.get(100, 200).unwrap();
        assert_eq!(result1.is_empty(), false);
    }

    #[test]
    fn test_get() {
        let mut db = RangeDb::open("test");
        assert_eq!(db.put(0, 100, b"Alice is owner").is_ok(), true);
        assert_eq!(db.put(100, 120, b"Bob is owner").is_ok(), true);
        assert_eq!(db.put(120, 180, b"Carol is owner").is_ok(), true);
        let result1 = db.get(20, 50).unwrap();
        assert_eq!(result1.is_empty(), true);
    }

}
