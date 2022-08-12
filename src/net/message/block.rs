//! # Block
//!
//! This module defines the block message structure

use crate::blockchain::Block as ChainBlock;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Block {
    pub block: ChainBlock,
}

impl Block {
    pub fn new(block: ChainBlock) -> Self {
        Self { block }
    }
}
