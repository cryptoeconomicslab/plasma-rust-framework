use crate::error::Error;
use crate::range::Range;

pub trait RangeStore {
    /// get ranges between start and end
    fn get(&self, start: u64, end: u64) -> Result<Vec<Range>, Error>;
    /// delete ranges between start and end
    fn del(&self, start: u64, end: u64) -> Result<Vec<Range>, Error>;
    /// put a range in start and end
    fn put(&mut self, start: u64, end: u64, value: &[u8]) -> Result<(), Error>;
}