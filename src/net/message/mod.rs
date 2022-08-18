//! # Msg
//!
//! This module expose the different Messages supported by the P2P network

mod block;
mod miners;
mod request_block;
mod transaction;
mod wallet;

use crate::{
    blockchain::{Block as ChainBlock, Transaction as BlockchainTransaction},
    mining::Miner,
};

use block::Block;
use miners::RegisteredMiners;
use request_block::RequestBlock;
use rust_decimal::Decimal;
pub use transaction::{
    Transaction, TransactionError, TransactionErrorCode, TransactionResult, TransactionStatus,
};
pub use wallet::{WalletQuery, WalletQueryError, WalletQueryResult, WalletTransactions};

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
    /// A message sent by a client to perform a transaction
    Transaction(Transaction),
    /// A message sent back to the client, with the result of the transaction
    TransactionResult(TransactionResult),
    /// A message sent by the client to request the wallet amount and the list of transactions for a certain address
    WalletDetails(WalletQuery),
    /// A message sent by a node to the client with the amount of the requested wallet
    WalletDetailsResult(WalletQueryResult),
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

    /// Create a `Transaction` message
    pub fn transaction(
        peer_id: impl ToString,
        input_address: impl ToString,
        output_address: impl ToString,
        amount: Decimal,
        public_key: impl ToString,
        signature: impl ToString,
    ) -> Self {
        Self::Transaction(Transaction::new(
            peer_id,
            input_address,
            output_address,
            amount,
            public_key,
            signature,
        ))
    }

    /// Create a successful `TransactionResult` message
    pub fn transaction_result_ok() -> Self {
        Self::TransactionResult(TransactionResult::new(TransactionStatus::Ok, None))
    }

    /// Create a `TransactionResult` with error message
    pub fn transaction_result_nok(code: TransactionErrorCode, description: impl ToString) -> Self {
        Self::TransactionResult(TransactionResult::new(
            TransactionStatus::Nok,
            Some(TransactionError::new(code, description)),
        ))
    }

    /// Create a `WalletDetails` message
    pub fn wallet_details(peer_id: impl ToString, address: impl ToString) -> Self {
        Self::WalletDetails(WalletQuery::new(peer_id, address))
    }

    /// Create a OK variant of a `WalletDetailsResult` message
    pub fn wallet_details_result_ok(
        address: impl ToString,
        transactions: Vec<BlockchainTransaction>,
        balance: Decimal,
    ) -> Self {
        Self::WalletDetailsResult(WalletQueryResult::ok(address, transactions, balance))
    }

    /// Create a ERROR variant of a `WalletDetailsResult` message
    pub fn wallet_details_result_error(error: WalletQueryError) -> Self {
        Self::WalletDetailsResult(WalletQueryResult::error(error))
    }
}
