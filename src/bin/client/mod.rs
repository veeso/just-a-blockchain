//! # Client application
//!
//! This module exposes the main client application

use std::path::Path;

use crate::Args;

use jab::blockchain::{Chain, TransactionVersion};
use jab::wallet::{Wallet, SECRET_KEY_SIZE};
use merkle::Hashable;
use ring::digest::{Context, SHA256};
use rust_decimal_macros::dec;
use std::fs;
use std::io::{Read, Write};

const WALLET_PUBLIC_KEY: &str = "jab.pub";
const WALLET_SECRET_KEY: &str = ".jab.key";

/// Defines the task to run in the client app
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Task {
    GenerateNewWallet,
    SignGenesisBlock,
    None,
}

pub struct App;

impl App {
    /// run client wallet
    pub fn run(task: Task, args: Args) -> anyhow::Result<()> {
        match task {
            Task::GenerateNewWallet => Self::generate_new_wallet(&args.wallet),
            Task::SignGenesisBlock => Self::sign_genesis_block(&args.wallet),
            Task::None => Ok(()),
        }
    }

    /// generate new wallet for client
    fn generate_new_wallet(p: &Path) -> anyhow::Result<()> {
        let wallet = Wallet::new();
        // create directory
        if let Err(err) = fs::create_dir_all(p) {
            anyhow::bail!("could not create directory at {}: {}", p.display(), err);
        }
        // write keys
        Self::write_key(p, WALLET_PUBLIC_KEY, wallet.public_key().as_bytes())?;
        Self::write_key(p, WALLET_SECRET_KEY, &wallet.secret_key())?;
        println!("created new wallet at {}", p.display());
        println!("your address is: {}", wallet.address());
        Ok(())
    }

    /// Sign genesis block
    fn sign_genesis_block(p: &Path) -> anyhow::Result<()> {
        let wallet = Self::open_wallet(p)?;
        let transaction =
            Chain::genesis_transaction(TransactionVersion::V1, wallet.address(), dec!(50.0))
                .finish("0");
        let mut digest_ctx = Context::new(&SHA256);
        transaction.update_context(&mut digest_ctx);
        let sha256 = digest_ctx.finish();
        let signature = wallet
            .sign(&sha256.as_ref())
            .map_err(|e| anyhow::anyhow!("failed to sign genesis transaction: {}", e))?;
        // verify signature is correct
        assert!(Wallet::verify(&sha256.as_ref(), &signature, &wallet.public_key()).unwrap());
        println!("genesis transaction signature: {}", signature);
        Ok(())
    }

    /// Open wallet located at `p`
    fn open_wallet(p: &Path) -> anyhow::Result<Wallet> {
        let secret_key = Self::read_key(p, WALLET_SECRET_KEY)?;
        Wallet::try_from(secret_key.as_slice())
            .map_err(|e| anyhow::anyhow!("failed to parse wallet: {}", e))
    }

    fn write_key(dir: &Path, filename: &str, key: &[u8]) -> anyhow::Result<()> {
        let mut p = dir.to_path_buf();
        p.push(filename);
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&p)
        {
            Ok(f) => f,
            Err(e) => anyhow::bail!("could not open file {}: {}", p.display(), e),
        };
        file.write_all(key)
            .map_err(|e| anyhow::anyhow!("failed to write key file {}: {}", p.display(), e))
    }

    fn read_key(dir: &Path, filename: &str) -> anyhow::Result<Vec<u8>> {
        let mut p = dir.to_path_buf();
        p.push(filename);
        let mut file = match fs::OpenOptions::new().read(true).open(&p) {
            Ok(f) => f,
            Err(err) => anyhow::bail!("could not open file {}: {}", p.display(), err),
        };
        let mut key_buffer = vec![0; SECRET_KEY_SIZE];
        file.read(key_buffer.as_mut_slice())
            .map_err(|e| anyhow::anyhow!("failed to read key file {}: {}", p.display(), e))?;
        Ok(key_buffer)
    }
}
