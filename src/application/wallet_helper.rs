//! Wallet helper
//!
//! An helper to create or initialize an existing wallet

use jab::wallet::{Wallet, SECRET_KEY_SIZE};
use std::path::Path;
use tokio::{fs::OpenOptions, io::AsyncReadExt};

pub struct WalletHelper;

impl WalletHelper {
    /// Open an existing wallet, or if it doesn't exist, create it
    pub async fn open_wallet(secret_key_path: &Path) -> anyhow::Result<Wallet> {
        if secret_key_path.exists() {
            let mut secret_key = OpenOptions::new()
                .read(true)
                .open(secret_key_path)
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "failed to open secret key path {}: {}",
                        secret_key_path.display(),
                        e
                    )
                })?;
            let mut buffer = vec![0; SECRET_KEY_SIZE];
            secret_key.read_exact(&mut buffer).await.map_err(|e| {
                anyhow::anyhow!(
                    "failed to read secret key from {}: {}",
                    secret_key_path.display(),
                    e
                )
            })?;
            Wallet::try_from(buffer.as_slice())
                .map_err(|e| anyhow::anyhow!("invalid wallet key: {}", e))
                .map(|w| {
                    info!("opened wallet with address {}", w.address());
                    w
                })
        } else {
            anyhow::bail!("wallet doesn't exist; please register a new wallet first");
        }
    }
}
