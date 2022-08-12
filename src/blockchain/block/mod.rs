//! # Block
//!
//! block module exposes the Block type and the block components

mod header;
mod transaction;

pub use header::{Header, Version};
pub use transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Block {
    /// Block index
    index: u64,
    /// Block header
    header: Header,
    /// Transaction information
    txns: Transaction,
}

impl Block {
    /// Instantiates a new `Block`
    pub fn new(index: u64, header: Header, txns: Transaction) -> Self {
        Self {
            index,
            header,
            txns,
        }
    }

    /// Return block index
    pub fn index(&self) -> u64 {
        self.index
    }

    /// Return a reference to the block header
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Return a reference to the block transaction
    pub fn transaction(&self) -> &Transaction {
        &self.txns
    }
}
