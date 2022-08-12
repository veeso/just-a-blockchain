//! # Errors
//!
//! This module defines the errors for the blockchain module

use crate::bridge::leveldb::LevelDbError;

use thiserror::Error;

/// Blockchain result type
pub type BlockchainResult<T> = Result<T, BlockchainError>;

#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("the block is invalid")]
    InvalidBlock,
    #[error("database error: {0}")]
    Database(LevelDbError),
    #[error("block in database has a bad value: {0}")]
    Json(serde_json::Error),
}

impl From<LevelDbError> for BlockchainError {
    fn from(e: LevelDbError) -> Self {
        Self::Database(e)
    }
}

impl From<serde_json::Error> for BlockchainError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
