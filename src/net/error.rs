//! # error
//!
//! exposes result and error types for node

use libp2p::{noise::NoiseError, TransportError};
use thiserror::Error;

/// Node result
pub type NodeResult<T> = Result<T, NodeError>;

/// Node error
#[derive(Error, Debug)]
pub enum NodeError {
    #[error("io error: {0}")]
    Io(std::io::Error),
    #[error("invalid payload codec: {0}")]
    InvalidPayload(serde_json::Error),
    #[error("noise error: {0}")]
    Noise(NoiseError),
    #[error("transport error: {0}")]
    TransportError(TransportError<std::io::Error>),
}

impl From<serde_json::Error> for NodeError {
    fn from(e: serde_json::Error) -> Self {
        Self::InvalidPayload(e)
    }
}

impl From<NoiseError> for NodeError {
    fn from(e: NoiseError) -> Self {
        Self::Noise(e)
    }
}

impl From<std::io::Error> for NodeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<TransportError<std::io::Error>> for NodeError {
    fn from(e: TransportError<std::io::Error>) -> Self {
        Self::TransportError(e)
    }
}
