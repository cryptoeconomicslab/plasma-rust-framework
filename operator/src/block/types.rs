use bytes::{BigEndian, ByteOrder, Bytes};
pub use num_traits::Zero;
use plasma_db::traits::kvs::BaseDbKey;
use std::ops::Add;

#[derive(Debug, Clone, Copy)]
pub struct BlockNumber(u64);

impl BlockNumber {
    pub fn new(n: u64) -> Self {
        Self(n)
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

impl From<BlockNumber> for BaseDbKey {
    fn from(block_number: BlockNumber) -> Self {
        let buf: [u8; 8] = block_number.into();
        BaseDbKey::from(&buf[..])
    }
}
