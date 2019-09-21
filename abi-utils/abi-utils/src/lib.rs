pub mod abi;
pub mod error;

pub use abi::{Decodable, Encodable};
use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use error::{Error, ErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub struct Integer(pub u64);

impl Integer {
    pub fn new(n: u64) -> Self {
        Integer(n)
    }
}

impl From<Integer> for Bytes {
    fn from(i: Integer) -> Self {
        let mut buf = BytesMut::with_capacity(64);
        buf.put_u64_le(i.0);
        Bytes::from(buf.to_vec())
    }
}

impl From<Bytes> for Integer {
    fn from(bytes: Bytes) -> Self {
        let mut buf = std::io::Cursor::new(bytes.to_vec());
        Integer(buf.get_u64_le())
    }
}
