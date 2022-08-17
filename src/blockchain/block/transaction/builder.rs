//! Transaction builder
//!
//! Used to SAFELY create transactions

use super::{LockOutput, Transaction, TransactionVersion, UnlockInput};

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

    /// Finish builder with signature
    pub fn finish(mut self, signature: impl ToString) -> Transaction {
        Transaction::new(
            self.version,
            self.inputs,
            self.outputs,
            signature.to_string(),
        )
    }
}
