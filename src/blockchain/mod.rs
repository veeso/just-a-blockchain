//! # Blockchain
//!
//! Blockchain module expose all the layers concerning the blockchain implementation

// -- modules
mod block;
mod database;
mod errors;
mod merkle;

use self::merkle::JabMerkleTree;
pub use block::{Block, Header, Transaction, TransactionBuilder, TransactionVersion, Version};
use database::BlockchainDatabase;
pub use errors::{BlockchainError, BlockchainResult};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const GENESIS_BLOCK_ADDRESS: &str = "jabbe2cce18177f64c3eb2cc51f0bd640dec8b22668";
const GENESIS_BLOCK_SIGNATURE: &str = "3045022100a6a9106ecbef322e967438dbc8f1bf0ea8f5ee75cd3519f55e2bb90693d67ee3022042ecad494ead5fd441814201e8ae915a934c29644984cfc3624e48290054a155";

/// The main blockchain struct, contains the entire blockchain and the methods to interact with it
pub struct Chain {
    /// the database which stores the blockchain
    blockchain: BlockchainDatabase,
}

impl TryFrom<&Path> for Chain {
    type Error = BlockchainError;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        // setup database
        let database = BlockchainDatabase::try_from(path)?;
        debug!("leveldb successfully initialized");
        // initialize database if genesis block doesn't exist
        if database.get_block(0)?.is_none() {
            debug!("database doesn't contain the genesis block yet; generating genesis block...");
            database.put_block(&Self::genesis_block())?;
            debug!("generated genesis block");
        }
        Ok(Self {
            blockchain: database,
        })
    }
}

impl Chain {
    /// Get genesis block (first block in the blockchain)
    pub fn get_genesis_block(&self) -> BlockchainResult<Block> {
        self.blockchain.get_block(0).map(|x| x.unwrap())
    }

    /// Push new block to the end of the blockchain
    pub fn add_block(&mut self, b: Block) -> BlockchainResult<()> {
        let previous_block = self.get_latest_block()?;
        if previous_block.index() < b.index()
            && b.header().previous_block_header_hash()
                == Some(previous_block.header().merkle_root_hash())
        {
            self.blockchain.put_block(&b)
        } else {
            Err(BlockchainError::InvalidBlock)
        }
    }

    /// Get block at `index`
    pub fn get_block(&self, index: u64) -> BlockchainResult<Option<Block>> {
        self.blockchain.get_block(index)
    }

    /// Get latest block. Unwrap is safe, since blockchain cannot be empty
    pub fn get_latest_block(&self) -> BlockchainResult<Block> {
        let mut index = 1;
        let mut block = self.get_genesis_block()?;
        loop {
            // if next block exists, update block and keep iterating; otherwise return last block `block`
            match self.get_block(index)? {
                None => break,
                Some(b) => block = b,
            }
            index += 1;
        }
        Ok(block)
    }

    /// Generate the next block in the blockchain
    pub fn generate_next_block(&mut self, transaction: Transaction) -> BlockchainResult<Block> {
        let previous_block = self.get_latest_block()?;
        let next_index = previous_block.index() + 1;
        let next_merkle_root = self.calc_merkle_root_hash()?;

        // generate new block
        let new_block = Block::new(
            next_index,
            Header::new(
                Version::V010,
                Some(previous_block.header().merkle_root_hash().to_string()),
                next_merkle_root,
                SystemTime::now(),
            ),
            transaction,
        );
        // add block and return latest block
        self.add_block(new_block)?;
        self.get_latest_block()
    }

    /// Get current jab amount for provided wallet
    pub fn wallet_amount(&self, addr: &str) -> BlockchainResult<Option<Decimal>> {
        let mut index = 0;
        let mut wallet_amount = Decimal::ZERO;
        let mut wallet_found = false;
        while let Some(block) = self.get_block(index)? {
            if block.transaction().input_address() == Some(addr) {
                // sum received money and sub spent money
                wallet_amount += block.transaction().amount_received(addr);
                wallet_amount -= block.transaction().amount_spent(addr);
                wallet_found = true;
            }
            index += 1;
        }
        if wallet_found {
            Ok(Some(wallet_amount))
        } else {
            Ok(None)
        }
    }

    /// Returns whether a certain wallet exists
    pub fn wallet_exists(&self, addr: &str) -> BlockchainResult<bool> {
        let mut index = 0;
        while let Some(block) = self.get_block(index)? {
            if block.transaction().input_address() == Some(addr) {
                return Ok(true);
            }
            index += 1;
        }
        Ok(false)
    }

    #[inline]
    fn genesis_block() -> Block {
        let genesis_transaction =
            Self::genesis_transaction(TransactionVersion::V1, GENESIS_BLOCK_ADDRESS, dec!(500.0))
                .finish(GENESIS_BLOCK_SIGNATURE);
        let tree = JabMerkleTree::new(vec![genesis_transaction.clone()]);
        Block::new(
            0,
            Header::new(Version::V010, None, tree.root_hash(), UNIX_EPOCH),
            genesis_transaction,
        )
    }

    #[inline]
    /// Get genesis transaction
    pub fn genesis_transaction(
        version: TransactionVersion,
        address: &str,
        amount: rust_decimal::Decimal,
    ) -> TransactionBuilder {
        TransactionBuilder::new(version).output(address, amount)
    }

    /// Calculate the merkle root hash from all the transactions in the blockchain
    fn calc_merkle_root_hash(&self) -> BlockchainResult<String> {
        let mut transactions: Vec<Transaction> = Vec::new();
        let mut index = 0;
        while let Some(block) = self.get_block(index)? {
            transactions.push(block.transaction().clone());
            index += 1;
        }
        let tree = JabMerkleTree::new(transactions);
        Ok(tree.root_hash())
    }
}
