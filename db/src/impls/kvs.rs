pub mod global_memory;
/// LevelDB implementation for key value store
#[cfg(feature = "require-leveldb")]
pub mod leveldb;
/// Memory implementation for key value store
pub mod memory;

pub use self::global_memory::GlobalMemoryDb;
#[cfg(feature = "require-leveldb")]
pub use self::leveldb::CoreDb as CoreDbLevelDbImpl;
pub use self::memory::CoreDbMemoryImpl;
