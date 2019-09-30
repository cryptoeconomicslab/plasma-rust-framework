use crate::error::Error;
use crate::range::Range;

pub trait RangeStore {
    /// get ranges between start and end
    fn get(&self, start: u64, end: u64) -> Result<Box<[Range]>, Error>;
    /// delete ranges between start and end
    fn del(&self, start: u64, end: u64) -> Result<Box<[Range]>, Error>;
    /// put a range in start and end
    fn put(&self, start: u64, end: u64, value: &[u8]) -> Result<(), Error>;
    /// update ranges between start and end applying closure
    fn update(&self, start: u64, end: u64, f: Box<dyn Fn(Range) -> Vec<u8>>) -> Result<(), Error>;
}
