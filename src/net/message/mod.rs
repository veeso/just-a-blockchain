//! # Msg
//!
//! This module expose the different Messages supported by the P2P network

mod block;
mod request_block;

use crate::blockchain::Block as ChainBlock;

use block::Block;
use request_block::RequestBlock;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Msg {
    /// A message to request block with provided index
    RequestBlock(RequestBlock),
    /// A message which responds with a requested block
    Block(Block),
}

impl Msg {
    /// Create a `RequestBlock` message
    pub fn request_block(index: u64) -> Self {
        Self::RequestBlock(RequestBlock::new(index))
    }

    /// Create a `Block` message
    pub fn block(block: ChainBlock) -> Self {
        Self::Block(Block::new(block))
    }
}
