//! # Application configuration
//!
//! This module contains the configuration for the application

use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize)]
/// Application config
pub struct Config {
    database_directory: PathBuf,
    wallet_secret_key: PathBuf,
}

impl Config {
    /// Try to create config from env
    pub fn try_from_env() -> anyhow::Result<Self> {
        envy::from_env()
            .map_err(|e| anyhow::anyhow!("could not load config from environment: {}", e))
    }

    /// Get database directory path
    pub fn database_dir(&self) -> &Path {
        self.database_directory.as_path()
    }

    /// Get wallet directory
    pub fn wallet_secret_key(&self) -> &Path {
        &self.wallet_secret_key
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_load_config_from_env_devel() {
        dotenv::from_filename(Path::new(".env.devel")).ok();
        let config = Config::try_from_env().unwrap();
        assert_eq!(config.database_dir(), Path::new("./db"));
        assert_eq!(config.wallet_secret_key(), Path::new("wallet.key"));
    }
}
