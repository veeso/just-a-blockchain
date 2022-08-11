//! # Header
//!
//! block header

use sha2::{Digest, Sha256};
use std::{
    hash::Hash,
    time::{SystemTime, UNIX_EPOCH},
};

/// Blockchain version
#[derive(Debug, Hash)]
pub enum Version {
    V010,
}

impl ToString for Version {
    fn to_string(&self) -> String {
        String::from(match &self {
            Self::V010 => "V010",
        })
    }
}

/// Blockchain header
#[derive(Debug, Hash)]
pub struct Header {
    /// blockchain version
    version: Version,
    /// previous block's header SHA256 hash
    previous_block_header_hash: Option<String>,
    /// hash of the merkle tree root
    merkle_root_hash: String,
    /// the UNIX epoch time the miner started hashing the header
    created_at: SystemTime,
}

impl Header {
    /// Instantiates a new block `Header`
    pub fn new(
        version: Version,
        previous_block_header_hash: Option<String>,
        merkle_root_hash: String,
        created_at: SystemTime,
    ) -> Self {
        Self {
            version,
            previous_block_header_hash,
            merkle_root_hash,
            created_at,
        }
    }

    /// Calculate sha256 of header
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.version.to_string());
        if let Some(hash) = &self.previous_block_header_hash {
            hasher.update(hash);
        }
        hasher.update(&self.merkle_root_hash);
        hasher.update(
            self.created_at
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string(),
        );
        hex::encode(hasher.finalize())
    }

    /// Get previous block header hash
    pub fn previous_block_header_hash(&self) -> Option<&str> {
        self.previous_block_header_hash.as_deref()
    }

    /// Get merkle root hash
    pub fn merkle_root_hash(&self) -> &str {
        &self.merkle_root_hash
    }
}
