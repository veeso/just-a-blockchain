//! # Transaction helper
//!
//! An helper to commit transactions

use crate::blockchain::{
    BlockchainError, Chain, Transaction, TransactionBuilder, TransactionVersion,
};
use crate::net::message::TransactionErrorCode;
use crate::wallet::{Wallet, WalletError};

use merkle::Hashable;
use ring::digest::{Context, SHA256};
use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
/// Transaction rejected error
pub enum TransactionRejected {
    #[error("the requested amount could not be paid by the issuer")]
    InsufficientBalance,
    #[error("input wallet not found")]
    InputWalletNotFound,
    #[error("output wallet not found")]
    OutputWalletNotFound,
    #[error("transaction signature is invalid")]
    InvalidSignature,
    #[error("blockchain error: {0}")]
    BlockchainError(BlockchainError),
    #[error("wallet error: {0}")]
    WalletError(WalletError),
}

impl From<TransactionRejected> for TransactionErrorCode {
    fn from(e: TransactionRejected) -> Self {
        match e {
            TransactionRejected::BlockchainError(_) | TransactionRejected::WalletError(_) => {
                Self::BlockchainError
            }
            TransactionRejected::InputWalletNotFound => Self::InputWalletNotFound,
            TransactionRejected::InsufficientBalance => Self::InsufficientBalance,
            TransactionRejected::InvalidSignature => Self::InvalidSignature,
            TransactionRejected::OutputWalletNotFound => Self::OutputWalletNotFound,
        }
    }
}

/// Transaction helper
pub struct TransactionHelper;

impl TransactionHelper {
    /// Create transaction using the provided options
    pub async fn create_transaction(
        opts: TransactionOptions,
        wallet: &Wallet,
        blockchain: &Chain,
    ) -> Result<Transaction, TransactionRejected> {
        // Prevent negative amount
        if opts.amount < Decimal::ZERO {
            return Err(TransactionRejected::InsufficientBalance);
        }
        Self::check_wallet_amount(&opts.input_address, opts.amount, blockchain)?;
        Self::check_output(&opts.output_address, blockchain)?;
        // Calculate output amount; if amount is ZERO, keep zero (wallet creation)
        let output_amount = if opts.amount == Decimal::ZERO {
            Decimal::ZERO
        } else {
            opts.amount - opts.fee
        };
        // make transaction
        let transaction = TransactionBuilder::new(TransactionVersion::V1)
            .input(&opts.input_address, opts.amount)
            .output(&opts.output_address, output_amount)
            .output(wallet.address(), opts.fee)
            .finish(&opts.signature);
        // verify transaction signature
        Self::check_transaction_signature(&transaction, opts.signature.as_str())?;
        debug!(
            "transferring {} ({}) from {} to {} (fee: {})",
            output_amount, opts.amount, opts.input_address, opts.output_address, opts.fee
        );
        Ok(transaction)
    }

    /// Check whether input has enough jab to pay the transaction
    fn check_wallet_amount(
        addr: &str,
        amount: Decimal,
        blockchain: &Chain,
    ) -> Result<(), TransactionRejected> {
        match blockchain.wallet_amount(addr) {
            Ok(Some(wallet_amount)) if wallet_amount < amount => {
                Err(TransactionRejected::InsufficientBalance)
            }
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(TransactionRejected::InputWalletNotFound),
            Err(err) => Err(TransactionRejected::BlockchainError(err)),
        }
    }

    fn check_output(addr: &str, blockchain: &Chain) -> Result<(), TransactionRejected> {
        match blockchain.wallet_exists(addr) {
            Ok(true) => Ok(()),
            Ok(false) => Err(TransactionRejected::OutputWalletNotFound),
            Err(err) => Err(TransactionRejected::BlockchainError(err)),
        }
    }

    fn check_transaction_signature(
        transaction: &Transaction,
        pubkey: &str,
    ) -> Result<(), TransactionRejected> {
        let mut digest_ctx = Context::new(&SHA256);
        transaction.update_context(&mut digest_ctx);
        let sha256 = digest_ctx.finish();
        // verify signature is correct
        match Wallet::verify(sha256.as_ref(), transaction.signature(), pubkey) {
            Ok(true) => Ok(()),
            Ok(false) => Err(TransactionRejected::InvalidSignature),
            Err(err) => Err(TransactionRejected::WalletError(err)),
        }
    }
}

/// Transaction options
pub struct TransactionOptions {
    input_address: String,
    output_address: String,
    signature: String,
    public_key: String,
    amount: Decimal,
    fee: Decimal,
}

impl TransactionOptions {
    /// Initialize new transaction options
    pub fn new(input_address: impl ToString, output_address: impl ToString) -> Self {
        Self {
            input_address: input_address.to_string(),
            output_address: output_address.to_string(),
            public_key: String::default(),
            signature: String::default(),
            amount: Decimal::ZERO,
            fee: Decimal::ZERO,
        }
    }

    pub fn public_key(mut self, pubkey: impl ToString) -> Self {
        self.public_key = pubkey.to_string();
        self
    }

    pub fn signature(mut self, signature: impl ToString) -> Self {
        self.signature = signature.to_string();
        self
    }

    /// Set amount for the transaction
    pub fn amount(mut self, amount: Decimal) -> Self {
        self.amount = amount;
        self
    }

    /// Set fee for transaction
    pub fn fee(mut self, fee: Decimal) -> Self {
        self.fee = fee;
        self
    }
}
