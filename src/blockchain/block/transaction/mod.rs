//! # Transaction
//!
//! the transaction contained in the block

use merkle::Hashable;
use ring::digest::Context;

mod builder;
mod input;
mod output;

pub use builder::TransactionBuilder;
use input::UnlockInput;
use output::LockOutput;
use rust_decimal::Decimal;

/// Describes the transaction version
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TransactionVersion {
    V1 = 0x01,
}

/// The transaction, defines all the information exchanged in a transaction in the blockchain
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Transaction {
    version: TransactionVersion,
    /// Transaction inputs
    inputs: Vec<UnlockInput>,
    /// Transaction outputs
    outputs: Vec<LockOutput>,
    /// HEXLOWER encoded signature of the issuer. The message for the signature
    signature: String,
}

impl Transaction {
    /// instantiates a new `Transaction`
    fn new(
        version: TransactionVersion,
        inputs: Vec<UnlockInput>,
        outputs: Vec<LockOutput>,
        signature: String,
    ) -> Self {
        Self {
            version,
            inputs,
            outputs,
            signature,
        }
    }

    #[allow(dead_code)]
    /// Add input to transaction
    pub fn input(mut self, addr: impl ToString, amount: Decimal) -> Self {
        self.inputs.push(UnlockInput::new(addr, amount));
        self
    }

    #[allow(dead_code)]
    /// Add output to transaction
    pub fn output(mut self, addr: impl ToString, amount: Decimal) -> Self {
        self.outputs.push(LockOutput::new(addr, amount));
        self
    }

    /// Get the transaction signature
    pub fn signature(&self) -> &str {
        &self.signature
    }

    /// Get input address for transaction
    pub fn input_address(&self) -> Option<&str> {
        self.inputs.get(0).map(|x| x.address.as_str())
    }

    /// Returns the amount spent by `addr` in this transaction
    /// The number returned is ZERO or NEGATIVE by design
    pub fn amount_spent(&self, addr: &str) -> Decimal {
        let mut amount = Decimal::ZERO;
        for input in self.inputs.iter().filter(|x| x.address.as_str() == addr) {
            amount -= input.amount;
        }
        assert!(amount <= Decimal::ZERO);
        amount
    }

    /// Returns the amount received by `addr` after this transaction
    /// The amount returned is ZERO or POSITIVE by design
    pub fn amount_received(&self, addr: &str) -> Decimal {
        let mut amount = Decimal::ZERO;
        for output in self.outputs.iter().filter(|x| x.address.as_str() == addr) {
            amount += output.amount;
        }
        assert!(amount >= Decimal::ZERO);
        amount
    }
}

impl Hashable for Transaction {
    fn update_context(&self, context: &mut Context) {
        context.update(&[self.version as u8]);
        for input in &self.inputs {
            input.update_context(context);
        }
        for output in &self.outputs {
            output.update_context(context);
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn should_build_a_transaction() {
        let transaction = TransactionBuilder::new(TransactionVersion::V1)
            .input("alice", dec!(10.52))
            .output("bob", dec!(10.50))
            .output("miner", dec!(0.02))
            .finish("aaa");
        assert_eq!(transaction.version, TransactionVersion::V1);
        assert_eq!(transaction.inputs.len(), 1);
        assert_eq!(transaction.outputs.len(), 2);
    }

    #[test]
    fn should_correctly_calculate_input_amount() {
        let transaction = TransactionBuilder::new(TransactionVersion::V1)
            .input("alice", dec!(6.0))
            .input("alice", dec!(4.52))
            .output("bob", dec!(10.50))
            .output("miner", dec!(0.02))
            .finish("aaa");
        assert_eq!(transaction.amount_spent("alice"), dec!(-10.52));
        assert_eq!(transaction.amount_spent("bob"), Decimal::ZERO);
        assert_eq!(transaction.amount_spent("miner"), Decimal::ZERO);
    }

    #[test]
    fn should_correctly_calculate_output_amount() {
        let transaction = TransactionBuilder::new(TransactionVersion::V1)
            .input("alice", dec!(6.0))
            .input("alice", dec!(4.52))
            .output("bob", dec!(10.50))
            .output("miner", dec!(0.02))
            .finish("aaa");
        assert_eq!(transaction.amount_received("alice"), Decimal::ZERO);
        assert_eq!(transaction.amount_received("bob"), dec!(10.50));
        assert_eq!(transaction.amount_received("miner"), dec!(0.02));
    }
}
