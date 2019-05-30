use crate::error::Error;
use crate::range::Range;

pub trait RangeStore {
    fn get(&self, start: u64, end: u64) -> Result<Vec<Range>, Error>;
    fn del(&self, start: u64, end: u64) -> Result<Vec<Range>, Error>;
    fn put(&mut self, start: u64, end: u64, value: &[u8]) -> Result<(), Error>;
}
