//! # Input
//!
//! This module implements the transaction input

use merkle::Hashable;
use ring::digest::Context;
use rust_decimal::Decimal;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct UnlockInput {
    /// Address of the input wallet
    pub address: String,
    /// Amount spent by the input address
    pub amount: Decimal,
}

impl UnlockInput {
    /// Instantiate a new `UnlockInput`
    pub fn new(address: impl ToString, amount: Decimal) -> Self {
        Self {
            address: address.to_string(),
            amount,
        }
    }
}

impl Hashable for UnlockInput {
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
    fn should_create_a_new_input() {
        let input = UnlockInput::new("jab0930f5dfeba62bd8929846bbf0f1a08e995e37f1", Decimal::ONE);
        assert_eq!(
            input.address.as_str(),
            "jab0930f5dfeba62bd8929846bbf0f1a08e995e37f1"
        );
        assert_eq!(input.amount.to_string().as_str(), "1");
    }
}
