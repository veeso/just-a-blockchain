//! # Errors
//!
//! Wallet error types

use secp256k1::Error as Secp256k1Error;
use thiserror::Error;

/// Result returned by the wallet
pub type WalletResult<T> = Result<T, WalletError>;

#[derive(Debug, Error)]
/// Describes a wallet error
pub enum WalletError {
    #[error("secp256k1 error: {0}")]
    Secp256k1(Secp256k1Error),
}

impl From<Secp256k1Error> for WalletError {
    fn from(e: Secp256k1Error) -> Self {
        Self::Secp256k1(e)
    }
}
