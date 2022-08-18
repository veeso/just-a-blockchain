//! Transaction builder
//!
//! Used to SAFELY create transactions

use super::{LockOutput, Transaction, TransactionVersion, UnlockInput};
use crate::wallet::{Wallet, WalletError};

use merkle::Hashable;
use ring::digest::{Context, SHA256};
use rust_decimal::Decimal;

/// A safe builder to create transactions
pub struct TransactionBuilder {
    /// Transaction inputs
    inputs: Vec<UnlockInput>,
    /// Transaction outputs
    outputs: Vec<LockOutput>,
    /// Transaction version
    version: TransactionVersion,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new(version: TransactionVersion) -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
            version,
        }
    }

    /// Add input to transaction
    pub fn input(mut self, addr: impl ToString, amount: Decimal) -> Self {
        self.inputs.push(UnlockInput::new(addr, amount));
        self
    }

    /// Add output to transaction
    pub fn output(mut self, addr: impl ToString, amount: Decimal) -> Self {
        self.outputs.push(LockOutput::new(addr, amount));
        self
    }

    /// Sign transaction with wallet and return transaction
    pub fn sign_with_wallet(self, wallet: &Wallet) -> Result<Transaction, WalletError> {
        let mut transaction =
            Transaction::new(self.version, self.inputs, self.outputs, String::default());
        let mut digest_ctx = Context::new(&SHA256);
        transaction.update_context(&mut digest_ctx);
        let sha256 = digest_ctx.finish();
        let signature = wallet.sign(sha256.as_ref())?;
        transaction.signature = signature;
        Ok(transaction)
    }

    /// Finish builder with signature
    pub fn finish(self, signature: impl ToString) -> Transaction {
        Transaction::new(
            self.version,
            self.inputs,
            self.outputs,
            signature.to_string(),
        )
    }
}
