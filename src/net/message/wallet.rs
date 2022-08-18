//! # Wallet
//!
//! This module exposes the message types for wallet queries

use crate::blockchain::Transaction;

use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct WalletQuery {
    /// Id of the requesting peer
    pub peer_id: String,
    /// Address of the wallet to query
    pub address: String,
}

impl WalletQuery {
    /// Instantiate a new `WalletQuery`
    pub fn new(peer_id: impl ToString, address: impl ToString) -> Self {
        Self {
            peer_id: peer_id.to_string(),
            address: address.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
/// Wallet query result
pub enum WalletQueryResult {
    Ok(WalletTransactions),
    Error(WalletQueryError),
}

impl WalletQueryResult {
    /// Instantiate a `Ok` variant of `WalletQueryResult`
    pub fn ok(address: impl ToString, transactions: Vec<Transaction>, balance: Decimal) -> Self {
        Self::Ok(WalletTransactions {
            address: address.to_string(),
            transactions,
            balance,
        })
    }

    /// Instantiate a `Error` variant of `WalletQueryResult`
    pub fn error(err: WalletQueryError) -> Self {
        Self::Error(err)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
/// Transactions and balance for a certain wallet (OK response for `WalletQueryResult`)
pub struct WalletTransactions {
    /// Wallet address
    pub address: String,
    /// Transactions associated to this wallet
    pub transactions: Vec<Transaction>,
    /// Current wallet balance
    pub balance: Decimal,
}

#[derive(Error, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// Error type for query error
pub enum WalletQueryError {
    #[error("blockchain error")]
    BlockchainError,
    #[error("requested wallet could not be found")]
    WalletNotFound,
}
