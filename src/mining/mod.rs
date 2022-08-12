//! # Mining
//!
//! this module exposes mining information

mod miner;

pub use miner::Miner;

/// The mining database contains the current information regarding the network miners
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

    /// Get host miner
    pub fn host(&self) -> &Miner {
        self.miners.get(0).unwrap()
    }

    /// Get miner associated to the last block mined
    pub fn last_block_mined_by(&self) -> Option<&Miner> {
        self.last_block_mined_by
            .as_deref()
            .and_then(|x| self.miner_by_id(x))
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
        if let Some(index) = self.index_of(&id) {
            self.miners.remove(index);
            info!("{} unregistered from miners", id);
        }
    }

    /// Set last block miner
    pub fn set_last_block_miner(&mut self) {
        let mut index = 0;
        if let Some(last_block_miner) = self.last_block_mined_by() {
            let new_index = self.index_of(last_block_miner.id()).unwrap_or_default();
            index = match new_index + 1 > self.miners().len() - 1 {
                true => 0,
                false => new_index + 1,
            };
        }
        let id = self.at(index).unwrap().id().to_string();
        if self.miner_exists(id.as_str()) {
            info!("last block miner updated to {}", id);
            self.last_block_mined_by = Some(id);
        }
    }

    /// Get index of miner with `id`
    pub fn index_of(&self, id: impl ToString) -> Option<usize> {
        let id = id.to_string();
        self.miners.iter().position(|x| *x.id() == id)
    }

    /// Get miner at `index`
    pub fn at(&self, index: usize) -> Option<&Miner> {
        self.miners.get(index)
    }

    /// returns whether a miner with `id` exists in the current database
    fn miner_exists(&self, id: &str) -> bool {
        self.miners.iter().any(|x| x.id() == id)
    }

    /// Get miner by id
    fn miner_by_id(&self, id: &str) -> Option<&Miner> {
        self.miners.iter().find(|x| x.id() == id)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_miner_database() {
        let database = MiningDatabase::new(Miner::new("host"));
        assert!(database.last_block_mined_by().is_none());
        assert_eq!(database.miners().len(), 1);
        assert_eq!(database.miners().get(0).unwrap().id(), "host");
    }

    #[test]
    fn should_get_index_of_miner() {
        let mut database = MiningDatabase::new(Miner::new("host"));
        database.register_miner(Miner::new("omar"));
        assert_eq!(database.index_of("omar").unwrap(), 1);
        assert!(database.index_of("foooooo").is_none());
    }

    #[test]
    fn should_register_miner() {
        let mut database = MiningDatabase::new(Miner::new("host"));
        database.register_miner(Miner::new("omar"));
        assert_eq!(database.miners().len(), 2);
        assert_eq!(database.miners().get(1).unwrap().id(), "omar");
    }

    #[test]
    fn should_unregister_miner() {
        let mut database = MiningDatabase::new(Miner::new("host"));
        database.register_miner(Miner::new("omar"));
        assert_eq!(database.miners().len(), 2);
        database.unregister_miner("omar");
        assert_eq!(database.miners().len(), 1);
        assert_eq!(database.miners().get(0).unwrap().id(), "host");
    }

    #[test]
    fn should_not_register_duped_miner() {
        let mut database = MiningDatabase::new(Miner::new("host"));
        database.register_miner(Miner::new("omar"));
        database.register_miner(Miner::new("omar"));
        assert_eq!(database.miners().len(), 2);
    }

    #[test]
    fn should_not_unregister_unexisting_miner() {
        let mut database = MiningDatabase::new(Miner::new("host"));
        database.register_miner(Miner::new("omar"));
        assert_eq!(database.miners().len(), 2);
        database.unregister_miner("omar");
        assert_eq!(database.miners().len(), 1);
        database.unregister_miner("omar");
        assert_eq!(database.miners().len(), 1);
    }

    #[test]
    fn should_set_last_block_mined_by() {
        let mut database = MiningDatabase::new(Miner::new("host"));
        database.set_last_block_miner();
        assert_eq!(database.last_block_mined_by.as_deref().unwrap(), "host");
    }

    #[test]
    fn should_get_last_block_mined_by() {
        let mut database = MiningDatabase::new(Miner::new("host"));
        database.set_last_block_miner();
        assert_eq!(
            database.last_block_mined_by().unwrap(),
            database.miners().get(0).unwrap(),
        );
    }
}
