//! # Miner
//!
//! This module define the structure for a miner

// Describe a miner in the network
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Miner {
    id: String,
}

impl Miner {
    /// instantiates a new `Miner`
    pub fn new(id: impl ToString) -> Self {
        Self { id: id.to_string() }
    }

    /// get `Miner` id
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_miner() {
        let miner = Miner::new("foo");
        assert_eq!(miner.id(), "foo");
    }
}
