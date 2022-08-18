//! # Transaction
//!
//! This module defines the payload for a transaction

use rust_decimal::Decimal;
use thiserror::Error;

/// Transaction payload. Used to send money from a wallet to another
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Transaction {
    /// Id of the peer requesting the transaction. Used to send response
    pub peer_id: String,
    /// Origin address
    pub input_address: String,
    /// Destination wallet
    pub output_address: String,
    /// Amount to send
    pub amount: Decimal,
    /// Wallte public key
    pub public_key: String,
    /// Transaction signature
    pub signature: String,
}

impl Transaction {
    /// Instantiate a new `Transaction` message
    pub fn new(
        peer_id: impl ToString,
        input_address: impl ToString,
        output_address: impl ToString,
        amount: Decimal,
        public_key: impl ToString,
        signature: impl ToString,
    ) -> Self {
        Self {
            peer_id: peer_id.to_string(),
            input_address: input_address.to_string(),
            output_address: output_address.to_string(),
            amount,
            public_key: public_key.to_string(),
            signature: signature.to_string(),
        }
    }
}

/// Transaction result payload. Used to report a transaction result
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TransactionResult {
    status: TransactionStatus,
    error: Option<TransactionError>,
}

impl TransactionResult {
    pub fn new(status: TransactionStatus, error: Option<TransactionError>) -> Self {
        Self { status, error }
    }
}

/// Transaction result status
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Ok,
    Nok,
}

/// Transaction error
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TransactionError {
    code: TransactionErrorCode,
    description: String,
}

impl TransactionError {
    pub fn new(code: TransactionErrorCode, description: impl ToString) -> Self {
        Self {
            code,
            description: description.to_string(),
        }
    }
}

/// Transaction error code
#[derive(Error, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionErrorCode {
    #[error("input wallet could not be found")]
    InputWalletNotFound,
    #[error("output wallet could not be found")]
    OutputWalletNotFound,
    #[error("you don't have enough jab to perform this transaction")]
    InsufficientBalance,
    #[error("the transaction signature is invalid")]
    InvalidSignature,
    #[error("blockchain error")]
    BlockchainError,
}
