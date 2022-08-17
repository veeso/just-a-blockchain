//! # Transaction
//!
//! the transaction contained in the block

use merkle::Hashable;
use ring::digest::Context;

/// The transaction, defines all the information exchanged in a transaction in the blockchain
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Transaction {
    pub dummy: String,
}

impl Hashable for Transaction {
    fn update_context(&self, context: &mut Context) {
        context.update(self.dummy.as_bytes())
    }
}

#[cfg(test)]
impl Default for Transaction {
    fn default() -> Self {
        Self {
            dummy: String::from("cafebabe"),
        }
    }
}
