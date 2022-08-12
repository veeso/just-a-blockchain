//! # Msg
//!
//! This module expose the different Messages supported by the P2P network

mod block;
mod miners;
mod request_block;

use crate::{blockchain::Block as ChainBlock, mining::Miner};

use block::Block;
use miners::RegisteredMiners;
use request_block::RequestBlock;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Msg {
    /// A message to request block with provided index
    RequestBlock(RequestBlock),
    /// A message which responds with a requested block
    Block(Block),
    /// A message which informs other peers to register the following miners
    RegisterMiners(RegisteredMiners),
    /// Request to the other peers the current registered miners
    RequestRegisteredMiners,
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

    /// Create a `RegisterMiners` message
    pub fn register_miners(miners: &[Miner]) -> Self {
        Self::RegisterMiners(RegisteredMiners::new(miners))
    }

    /// Create a `RequestRegisteredMiners` message
    pub fn request_registered_miners() -> Self {
        Self::RequestRegisteredMiners
    }
}
