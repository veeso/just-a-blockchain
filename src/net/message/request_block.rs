//! # RequestBlock
//!
//! This module expose the REQUEST_BLOCK message type

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RequestBlock {
    pub index: u64,
}

impl RequestBlock {
    pub fn new(index: u64) -> Self {
        Self { index }
    }
}
