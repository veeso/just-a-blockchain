//! # Output
//!
//! This module implements the transaction output

use merkle::Hashable;
use ring::digest::Context;
use rust_decimal::Decimal;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct LockOutput {
    /// Address of the output wallet
    pub address: String,
    /// Amount received by the input address
    pub amount: Decimal,
}

impl LockOutput {
    /// Instantiate a new `UnlockInput`
    pub fn new(address: impl ToString, amount: Decimal) -> Self {
        Self {
            address: address.to_string(),
            amount,
        }
    }
}

impl Hashable for LockOutput {
    fn update_context(&self, context: &mut Context) {
        context.update(self.address.as_bytes());
        context.update(self.amount.to_string().as_bytes());
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_a_new_output() {
        let output = LockOutput::new("jab0930f5dfeba62bd8929846bbf0f1a08e995e37f1", Decimal::ONE);
        assert_eq!(
            output.address.as_str(),
            "jab0930f5dfeba62bd8929846bbf0f1a08e995e37f1"
        );
        assert_eq!(output.amount.to_string().as_str(), "1");
    }
}
