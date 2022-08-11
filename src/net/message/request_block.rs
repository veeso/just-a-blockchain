//! # RequestBlock
//!
//! This module expose the REQUEST_BLOCK message type

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RequestBlock {
    pub index: u128,
}

impl RequestBlock {
    pub fn new(index: u128) -> Self {
        Self { index }
    }
}
