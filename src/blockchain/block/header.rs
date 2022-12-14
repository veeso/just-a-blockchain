//! # Header
//!
//! block header

use std::{str::FromStr, time::SystemTime};

/// Blockchain version
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Version {
    V010,
}

impl FromStr for Version {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "V010" => Ok(Self::V010),
            _ => Err("invalid version"),
        }
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        String::from(match &self {
            Self::V010 => "V010",
        })
    }
}

/// Blockchain header
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
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

    /// Get previous block header hash
    pub fn previous_block_header_hash(&self) -> Option<&str> {
        self.previous_block_header_hash.as_deref()
    }

    /// Get merkle root hash
    pub fn merkle_root_hash(&self) -> &str {
        &self.merkle_root_hash
    }
}
