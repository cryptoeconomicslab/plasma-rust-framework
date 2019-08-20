use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
pub use num_traits::{
    Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedShl, CheckedShr, CheckedSub, One,
    Saturating, Zero,
};
use std::fmt::Debug;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Shl, Shr, Sub, SubAssign,
};

pub trait LittleEndianEncodable {
    fn encode_as_le(self) -> Vec<u8>;
}

impl LittleEndianEncodable for u64 {
    fn encode_as_le(self) -> Vec<u8> {
        let mut end_writer = vec![];
        end_writer.write_u64::<LittleEndian>(self).unwrap();
        end_writer
    }
}

pub trait LittleEndianDecoder {
    fn decode_as_le(encoded: &[u8]) -> Self;
}

impl LittleEndianDecoder for u64 {
    fn decode_as_le(encoded: &[u8]) -> u64 {
        let mut reader = std::io::Cursor::new(encoded);
        reader.read_u64::<LittleEndian>().unwrap()
    }
}

pub trait Index:
    Zero
    + One
    + LittleEndianEncodable
    + LittleEndianDecoder
    + Add<Self, Output = Self>
    + AddAssign<Self>
    + Sub<Self, Output = Self>
    + SubAssign<Self>
    + Mul<Self, Output = Self>
    + MulAssign<Self>
    + Div<Self, Output = Self>
    + DivAssign<Self>
    + Rem<Self, Output = Self>
    + RemAssign<Self>
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + CheckedShl
    + CheckedShr
    + CheckedAdd
    + CheckedSub
    + CheckedMul
    + CheckedDiv
    + Saturating
    + PartialOrd<Self>
    + Ord
    + Bounded
    + Debug
    + Clone
    + Copy
{
}

impl<
        T: Zero
            + One
            + LittleEndianEncodable
            + LittleEndianDecoder
            + Add<Self, Output = Self>
            + AddAssign<Self>
            + Sub<Self, Output = Self>
            + SubAssign<Self>
            + Mul<Self, Output = Self>
            + MulAssign<Self>
            + Div<Self, Output = Self>
            + DivAssign<Self>
            + Rem<Self, Output = Self>
            + RemAssign<Self>
            + Shl<u32, Output = Self>
            + Shr<u32, Output = Self>
            + CheckedShl
            + CheckedShr
            + CheckedAdd
            + CheckedSub
            + CheckedMul
            + CheckedDiv
            + Saturating
            + PartialOrd<Self>
            + Ord
            + Bounded
            + Debug
            + Clone
            + Copy,
    > Index for T
{
}
