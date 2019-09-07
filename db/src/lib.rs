pub mod error;
pub mod impls;
pub mod range;
pub mod traits;

#[macro_use]
extern crate lazy_static;

pub use impls::rangedb::RangeDbImpl;
pub use traits::kvs::BaseDbKey;
