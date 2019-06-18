use bytes::{BigEndian, ByteOrder, Bytes};
pub use num_traits::Zero;
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockNumber(u64);

impl BlockNumber {
    pub fn new(n: u64) -> Self {
        Self(n)
    }
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl Zero for BlockNumber {
    fn zero() -> Self {
        Self(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}
impl Add for BlockNumber {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl From<Bytes> for BlockNumber {
    fn from(buffer: Bytes) -> Self {
        BlockNumber::new(BigEndian::read_u64(&buffer))
    }
}

impl From<Vec<u8>> for BlockNumber {
    fn from(buffer: Vec<u8>) -> Self {
        BlockNumber::new(BigEndian::read_u64(&buffer))
    }
}

impl From<BlockNumber> for Bytes {
    fn from(block_number: BlockNumber) -> Self {
        let mut buf = [0; 8];
        BigEndian::write_u64(&mut buf, block_number.0);
        Bytes::from(&buf[..])
    }
}

impl From<BlockNumber> for [u8; 8] {
    fn from(block_number: BlockNumber) -> Self {
        let mut buf = [0; 8];
        BigEndian::write_u64(&mut buf, block_number.0);
        buf
    }
}

impl From<BlockNumber> for u64 {
    fn from(block_number: BlockNumber) -> Self {
        block_number.0
    }
}

impl From<u64> for BlockNumber {
    fn from(n: u64) -> Self {
        BlockNumber(n)
    }
}
