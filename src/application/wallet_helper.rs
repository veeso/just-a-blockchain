//! Wallet helper
//!
//! An helper to create or initialize an existing wallet

use std::path::Path;

use crate::wallet::Wallet;
use jab::wallet::SECRET_KEY_SIZE;
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
};

pub struct WalletHelper;

impl WalletHelper {
    /// Open an existing wallet, or if it doesn't exist, create it
    pub async fn open_or_create_wallet(secret_key_path: &Path) -> anyhow::Result<Wallet> {
        if secret_key_path.exists() {
            Self::open_wallet(secret_key_path).await
        } else {
            Self::create_wallet(secret_key_path).await
        }
    }

    /// Open an existing wallet
    async fn open_wallet(secret_key_path: &Path) -> anyhow::Result<Wallet> {
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
        secret_key.read_buf(&mut buffer).await.map_err(|e| {
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
    }

    /// create a brand new wallet
    async fn create_wallet(secret_key_path: &Path) -> anyhow::Result<Wallet> {
        debug!("wallet doesn't exist; generating a new wallet...");
        let wallet = Wallet::new();
        info!("generated new wallet with address: {}", wallet.address());
        let mut secret_key = OpenOptions::new()
            .write(true)
            .create(true)
            .open(secret_key_path)
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "failed to open secret key path {}: {}",
                    secret_key_path.display(),
                    e
                )
            })?;
        secret_key
            .write_all(&wallet.secret_key())
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "failed to write secret key to {}: {}",
                    secret_key_path.display(),
                    e
                )
            })?;
        Ok(wallet)
    }
}
