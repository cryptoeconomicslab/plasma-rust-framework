/// LevelDB implementation for key value store
#[cfg(leveldb)]
pub mod leveldb;
/// Memory implementation for key value store
pub mod memory;

#[cfg(leveldb)]
pub use self::leveldb::CoreDb as CoreDbLevelDbImpl;
pub use self::memory::CoreDbMemoryImpl;
