//! # Blockchain
//!
//! Blockchain module expose all the layers concerning the blockchain implementation

// -- modules
mod block;
mod merkle;

use std::time::UNIX_EPOCH;
use thiserror::Error;

use self::merkle::JabMerkleTree;
pub use block::{Block, Header, Transaction, Version};

/// Blockchain result type
pub type BlockchainResult<T> = Result<T, BlockchainError>;

#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("the block is invalid")]
    InvalidBlock,
}

/// The main blockchain struct, contains the entire blockchain and the methods to interact with it
#[derive(Debug)]
pub struct Chain {
    /// the entire blocks which composes the blockchain
    blockchain: Vec<Block>,
}

impl Chain {
    /// Initialize a new blockchain from scratch
    pub fn new() -> Self {
        Self {
            blockchain: vec![Self::genesis_block()],
        }
    }

    /// Get genesis block (first block in the blockchain)
    pub fn get_genesis_block(&self) -> &Block {
        self.blockchain.get(0).unwrap()
    }

    /// Push new block to the end of the blockchain
    pub fn add_block(&mut self, b: Block) -> BlockchainResult<()> {
        let previous_block = self.get_latest_block();
        if previous_block.index() < b.index()
            && b.header().previous_block_header_hash()
                == Some(previous_block.header().merkle_root_hash())
        {
            self.blockchain.push(b);
            Ok(())
        } else {
            Err(BlockchainError::InvalidBlock)
        }
    }

    /// Get block at `index`
    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.blockchain.get(index as usize)
    }

    /// Get latest block. Unwrap is safe, since blockchain cannot be empty
    pub fn get_latest_block(&self) -> &Block {
        self.blockchain.last().unwrap()
    }

    pub fn generate_next_block(&mut self) -> BlockchainResult<u64> {
        todo!()
    }

    #[inline]
    fn genesis_block() -> Block {
        let genesis_transaction = Transaction {
            dummy: String::from("foobar"),
        };
        let tree = JabMerkleTree::new(vec![genesis_transaction.clone()]);
        Block::new(
            0,
            Header::new(Version::V010, None, tree.root_hash(), UNIX_EPOCH),
            genesis_transaction,
        )
    }
}
