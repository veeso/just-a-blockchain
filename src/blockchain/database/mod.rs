//! # Database
//!
//! Database to store the blocks of our blockchain

mod key;

use super::{Block, BlockchainError, BlockchainResult};
use crate::bridge::leveldb::LevelDbBridge;
use key::BlockKey;

use std::path::Path;

/// Blockchain database client
pub struct BlockchainDatabase {
    database: LevelDbBridge<BlockKey>,
}

impl TryFrom<&Path> for BlockchainDatabase {
    type Error = BlockchainError;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        debug!("initializing blockchain database");
        Ok(Self {
            database: LevelDbBridge::init(path)?,
        })
    }
}

impl BlockchainDatabase {
    /// Put block into the database
    pub fn put_block(&self, block: &Block) -> BlockchainResult<()> {
        let payload = serde_json::json!(block).to_string();
        info!("inserting block {} ({})", block.index(), payload);
        self.database
            .put(block.index().into(), payload.as_bytes())
            .map_err(BlockchainError::from)
    }

    /// Get block from database with provided index
    pub fn get_block(&self, index: u64) -> BlockchainResult<Option<Block>> {
        debug!("getting block with index {}", index);
        self.database
            .get(index.into())?
            .map(|payload| serde_json::from_slice(&payload))
            .transpose()
            .map_err(|e| {
                error!(
                    "key with index {} has a bad payload; deleting it from database",
                    index
                );
                let _ = self.database.delete(index.into());
                BlockchainError::from(e)
            })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::blockchain::{Header, Transaction, Version};

    use pretty_assertions::assert_eq;
    use std::time::SystemTime;
    use tempfile::TempDir;

    #[test]
    fn should_put_blocks_in_the_leveldb() {
        let tempdir = TempDir::new().expect("could not create tempfile");
        let path = tempdir.path();
        let database = BlockchainDatabase::try_from(path).unwrap();

        // put block
        let block = Block::new(
            0,
            Header::new(
                Version::V010,
                None,
                String::from("cafebabe"),
                SystemTime::now(),
            ),
            Transaction::default(),
        );
        assert!(database.put_block(&block).is_ok());
        // get block
        assert_eq!(database.get_block(0).unwrap().unwrap(), block);
        // get unexisting block
        assert!(database.get_block(1).unwrap().is_none());
    }
}
