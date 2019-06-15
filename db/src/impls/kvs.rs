/// LevelDB implementation for key value store
#[cfg(not(leveldb))]
pub mod leveldb;
/// Memory implementation for key value store
pub mod memory;

#[cfg(not(leveldb))]
pub use self::leveldb::CoreDb as CoreDbLevelDbImpl;
pub use self::memory::CoreDbMemoryImpl;
