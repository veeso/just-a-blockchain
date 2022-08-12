//! # Mining
//!
//! this module exposes mining information

pub struct MiningDatabase {
    miners: Vec<Miner>,
    last_block_mined_by: Option<String>,
}

impl MiningDatabase {
    /// Instantiate a new `MiningDatabase`
    pub fn new(host_miner: Miner) -> Self {
        Self {
            miners: vec![host_miner],
            last_block_mined_by: None,
        }
    }

    /// Get miners
    pub fn miners(&self) -> &[Miner] {
        &self.miners
    }

    /// Add miner to miners list
    pub fn register_miner(&mut self, miner: Miner) {
        if !self.miner_exists(miner.id()) {
            info!("added new miner {}", miner.id());
            self.miners.push(miner);
        }
    }

    /// unregister miner
    pub fn unregister_miner(&mut self, id: impl ToString) {
        let id = id.to_string();
        debug!("unregistering miner {}", id);
        let index = self.miners.iter().position(|x| *x.id() == id);
        if let Some(index) = index {
            self.miners.remove(index);
            info!("{} unregistered from miners", id);
        }
    }

    /// Set last block miner
    pub fn set_last_block_miner(&mut self, id: impl ToString) {
        let id = id.to_string();
        if self.miner_exists(id.as_str()) {
            info!("last block miner updated to {}", id);
            self.last_block_mined_by = Some(id);
        }
    }

    /// returns whether a miner with `id` exists in the current database
    fn miner_exists(&self, id: &str) -> bool {
        self.miners.iter().any(|x| x.id() == id)
    }
}

// Describe a miner in the network
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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
