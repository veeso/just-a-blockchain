//! # Transaction
//!
//! the transaction contained in the block

use merkle::Hashable;
use ring::digest::Context;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Transaction {
    pub dummy: String,
}

impl Hashable for Transaction {
    fn update_context(&self, context: &mut Context) {
        context.update(self.dummy.as_bytes())
    }
}
