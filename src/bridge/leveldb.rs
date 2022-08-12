//! # LevelDB
//!
//! a bridge to interface with a leveldb database

pub use db_key::Key;
use leveldb::{
    database::Database,
    error::Error as DbError,
    kv::KV,
    options::{Options, ReadOptions, WriteOptions},
};
use std::path::Path;
use thiserror::Error;

/// The result type returned by an operation on the database
pub type LevelDbResult<T> = Result<T, LevelDbError>;

/// Describe an error on the level db
#[derive(Debug, Error)]
pub enum LevelDbError {
    #[error("database error: {0}")]
    Database(DbError),
}

impl From<DbError> for LevelDbError {
    fn from(e: DbError) -> Self {
        Self::Database(e)
    }
}

/// a bridge to operate on a levelDB
pub struct LevelDbBridge<K: Key> {
    database: Database<K>,
}

impl<K> LevelDbBridge<K>
where
    K: Key,
{
    /// Initialize level db bridge
    pub fn init<P>(path: P) -> LevelDbResult<Self>
    where
        P: AsRef<Path>,
    {
        let mut options = Options::new();
        options.create_if_missing = true;
        Database::open(path.as_ref(), options)
            .map(|x| Self { database: x })
            .map_err(LevelDbError::from)
    }

    /// Put key and value into the level database
    pub fn put(&self, key: K, value: &[u8]) -> LevelDbResult<()> {
        self.database
            .put(WriteOptions::new(), key, value)
            .map_err(LevelDbError::from)
    }

    /// Get `key` from database
    pub fn get(&self, key: K) -> LevelDbResult<Option<Vec<u8>>> {
        self.database
            .get(ReadOptions::new(), key)
            .map_err(LevelDbError::from)
    }

    /// Delete `key` from database
    pub fn delete(&self, key: K) -> LevelDbResult<()> {
        self.database
            .delete(WriteOptions::new(), key)
            .map_err(LevelDbError::from)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    #[test]
    fn should_open_leveldb() {
        let tempdir = TempDir::new().expect("could not create tempfile");
        let path = tempdir.path();
        let _: LevelDbBridge<i32> = LevelDbBridge::init(path).unwrap();
    }

    #[test]
    fn should_put_and_get_keys() {
        let tempdir = TempDir::new().expect("could not create tempfile");
        let path = tempdir.path();
        let database: LevelDbBridge<i32> = LevelDbBridge::init(path).unwrap();
        assert!(database.put(30, &[0x01]).is_ok());
        assert_eq!(database.get(30).unwrap().unwrap(), vec![0x01]);
        assert!(database.get(10).unwrap().is_none());
    }

    #[test]
    fn should_delete_key_from_database() {
        let tempdir = TempDir::new().expect("could not create tempfile");
        let path = tempdir.path();
        let database: LevelDbBridge<i32> = LevelDbBridge::init(path).unwrap();
        assert!(database.put(30, &[0x01]).is_ok());
        assert!(database.delete(30).is_ok());
        assert!(database.get(30).unwrap().is_none());
    }
}
