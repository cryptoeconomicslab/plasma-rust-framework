extern crate tempdir;

use crate::error::{Error, ErrorKind};
use crate::range::Range;
use crate::traits::kvs::{BaseDbKey, Batch, Bucket, KeyValueStore};
use crate::traits::rangestore::RangeStore;
use bytes::Bytes;

/// Range DB implementation using key value store.
/// ```rust
/// use plasma_db::impls::kvs::CoreDbMemoryImpl;
/// use plasma_db::RangeDbImpl;
/// use plasma_db::traits::db::DatabaseTrait;
/// let base_db = CoreDbMemoryImpl::open("test");
/// let db = RangeDbImpl::from(base_db);
/// ```
pub struct RangeDbImpl<KVS> {
    db: KVS,
}

impl<KVS> RangeDbImpl<KVS>
where
    KVS: KeyValueStore,
{
    pub fn bucket<'a>(&'a self, key: &Bytes) -> RangeDbImpl<Bucket<'a>> {
        RangeDbImpl {
            db: self.db.bucket(&BaseDbKey::from(key.clone())),
        }
    }

    fn validate_range(start: u64, end: u64) -> bool {
        start < end
    }
    pub fn del_batch(&self, start: u64, end: u64) -> Result<Box<[Range]>, Error> {
        let ranges = self.get(start, end)?;
        let mut batch = vec![];
        for range in ranges.clone().iter() {
            batch.push(Batch::new_del(BaseDbKey::from(range.get_end())));
        }
        if self.db.batch(&batch).is_ok() {
            Ok(ranges)
        } else {
            Err(Error::from(ErrorKind::LevelDb))
        }
    }
    pub fn put_batch(&self, ranges: &[Range]) -> Result<(), Error> {
        let mut batch = vec![];
        for range in ranges.iter() {
            batch.push(Batch::new_put(
                BaseDbKey::from(range.get_end()),
                &rlp::encode(range),
            ));
        }
        if self.db.batch(&batch).is_ok() {
            Ok(())
        } else {
            Err(Error::from(ErrorKind::LevelDb))
        }
    }
}

impl<KVS> From<KVS> for RangeDbImpl<KVS>
where
    KVS: KeyValueStore,
{
    fn from(kvs: KVS) -> Self {
        Self { db: kvs }
    }
}

impl<KVS> RangeStore for RangeDbImpl<KVS>
where
    KVS: KeyValueStore,
{
    fn get(&self, start: u64, end: u64) -> Result<Box<[Range]>, Error> {
        let result: Vec<Range> = self
            .db
            .iter_all(
                &BaseDbKey::from(start),
                Box::new(move |_k, v| {
                    let range: Range = rlp::decode(&v).unwrap();
                    range.intersect(start, end)
                }),
            )
            .iter()
            .filter_map(|kv| rlp::decode(kv.get_value()).ok())
            .collect();
        Ok(result.into_boxed_slice())
    }
    fn del(&self, start: u64, end: u64) -> Result<Box<[Range]>, Error> {
        self.del_batch(start, end)
    }
    fn put(&self, start: u64, end: u64, value: &[u8]) -> Result<(), Error> {
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
    use super::RangeDbImpl;
    use crate::impls::kvs::memory::CoreDbMemoryImpl;
    use crate::traits::db::DatabaseTrait;
    use crate::traits::rangestore::RangeStore;
    use bytes::Bytes;

    #[test]
    fn test_get_same_range() {
        let base_db = CoreDbMemoryImpl::open("test");
        let db = RangeDbImpl::from(base_db);
        assert_eq!(db.put(0, 100, b"Alice is owner").is_ok(), true);
        assert_eq!(db.put(100, 200, b"Bob is owner").is_ok(), true);
        let result1 = db.get(100, 200).unwrap();
        assert_eq!(result1.is_empty(), false);
        assert_eq!(result1[0].get_start(), 100);
        assert_eq!(result1[0].get_value(), b"Bob is owner");
    }

    #[test]
    fn test_get_small_range() {
        let base_db = CoreDbMemoryImpl::open("test");
        let db = RangeDbImpl::from(base_db);
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
        let base_db = CoreDbMemoryImpl::open("test");
        let db = RangeDbImpl::from(base_db);
        assert_eq!(db.put(0, 100, b"Alice is owner").is_ok(), true);
        assert_eq!(db.put(100, 120, b"Bob is owner").is_ok(), true);
        assert_eq!(db.put(120, 180, b"Carol is owner").is_ok(), true);
        let result1 = db.get(20, 150).unwrap();
        assert_eq!(result1.is_empty(), false);
        assert_eq!(result1[0].get_start(), 0);
        assert_eq!(result1[0].get_value(), b"Alice is owner");
        assert_eq!(result1.len(), 3);
    }

    #[test]
    fn test_bucket() {
        let base_db = CoreDbMemoryImpl::open("test");
        let db = RangeDbImpl::from(base_db);
        let bucket_name = Bytes::from("aaa");
        assert_eq!(
            db.bucket(&bucket_name)
                .put(0, 100, b"Alice is owner")
                .is_ok(),
            true
        );
        assert_eq!(
            db.bucket(&bucket_name)
                .put(100, 200, b"Bob is owner")
                .is_ok(),
            true
        );
        let result1 = db.bucket(&bucket_name).get(100, 200).unwrap();
        assert_eq!(result1.is_empty(), false);
        assert_eq!(result1[0].get_start(), 100);
        assert_eq!(result1[0].get_value(), b"Bob is owner");
    }

}
