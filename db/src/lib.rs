pub mod error;
pub mod impls;
pub mod range;
pub mod traits;

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "require-leveldb")]
pub use impls::kvs::CoreDbLevelDbImpl;
pub use impls::kvs::{CoreDbMemoryImpl, GlobalMemoryDb};
pub use impls::rangedb::RangeDbImpl;

pub use traits::db::DatabaseTrait;
pub use traits::kvs::{BaseDbKey, KeyValueStore};
pub use traits::rangestore::RangeStore;
