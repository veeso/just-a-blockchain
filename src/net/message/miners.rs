//! # Miners
//!
//! A message to send the known registered miners

use crate::mining::Miner;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct RegisteredMiners {
    pub miners: Vec<Miner>,
}

impl RegisteredMiners {
    pub fn new(miners: &[Miner]) -> Self {
        Self {
            miners: miners.to_vec(),
        }
    }
}
