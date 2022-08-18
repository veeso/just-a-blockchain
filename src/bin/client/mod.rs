//! # Client application
//!
//! This module exposes the main client application

use std::path::Path;

use crate::Args;

use futures::executor::block_on;
use futures::StreamExt;
use jab::blockchain::{Chain, Transaction, TransactionBuilder, TransactionVersion};
use jab::net::{
    message::{TransactionResult, TransactionStatus, WalletQueryResult, WalletTransactions},
    Msg, Node,
};
use jab::wallet::{Wallet, SECRET_KEY_SIZE};
use merkle::Hashable;
use ring::digest::{Context, SHA256};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fs;
use std::io::{Read, Write};
use std::str::FromStr;

const WALLET_PUBLIC_KEY: &str = "jab.pub";
const WALLET_SECRET_KEY: &str = ".jab.key";

/// Defines the task to run in the client app
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Task {
    GenerateNewWallet,
    SignGenesisBlock,
    GetBalance,
    GetBalanceFor(String),
    Send,
    None,
}

pub struct App;

impl App {
    /// run client wallet
    pub fn run(task: Task, args: Args) -> anyhow::Result<()> {
        match task {
            Task::GenerateNewWallet => Self::generate_new_wallet(&args.wallet),
            Task::GetBalance => Self::get_balance(&args.wallet),
            Task::GetBalanceFor(addr) => Self::get_balance_for(&addr),
            Task::Send => Self::send(&args.wallet),
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
        // publish wallet to blockchain
        let transaction = Self::make_transaction(&wallet, wallet.address(), Decimal::ZERO)?;
        let mut node = Self::start_p2p_node()?;
        Self::publish_transaction(&mut node, transaction, Decimal::ZERO, wallet.public_key())?;
        println!("created new wallet at {}", p.display());
        println!("your address is: {}", wallet.address());
        Ok(())
    }

    /// Get balance for this wallet
    fn get_balance(p: &Path) -> anyhow::Result<()> {
        let wallet = Self::open_wallet(p)?;
        Self::get_balance_for(wallet.address())
    }

    /// Get balance for provided address
    fn get_balance_for(address: &str) -> anyhow::Result<()> {
        let mut node = Self::start_p2p_node()?;
        let (balance, transactions) = Self::publish_get_balance(&mut node, address)?;
        for transaction in transactions.into_iter() {
            println!(
                "{} {}JAB => {} {}JAB",
                transaction.input_address().unwrap_or_default(),
                transaction.amount_spent(transaction.input_address().unwrap_or_default()),
                transaction.output_address().unwrap_or_default(),
                transaction.amount_received(transaction.output_address().unwrap_or_default()),
            );
        }
        println!("wallet amount for {}: {}", address, balance);
        Ok(())
    }

    /// Send money from this wallet to another
    fn send(p: &Path) -> anyhow::Result<()> {
        let wallet = Self::open_wallet(p)?;
        // ask for receiver wallet
        println!("Enter recipient wallet :");
        let mut recipient = String::new();
        std::io::stdin().read_line(&mut recipient).unwrap();
        // ask amount to send
        println!("Enter amount to send :");
        let mut amount = String::new();
        std::io::stdin().read_line(&mut amount).unwrap();
        let amount =
            Decimal::from_str(&amount).map_err(|e| anyhow::anyhow!("bad amount: {}", e))?;
        // send
        let transaction = Self::make_transaction(&wallet, &recipient, amount)?;
        let mut node = Self::start_p2p_node()?;
        Self::publish_transaction(&mut node, transaction, amount, wallet.public_key())?;
        println!("sent {} to {}", amount, recipient);
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
            .sign(sha256.as_ref())
            .map_err(|e| anyhow::anyhow!("failed to sign genesis transaction: {}", e))?;
        // verify signature is correct
        assert!(Wallet::verify(sha256.as_ref(), &signature, &wallet.public_key()).unwrap());
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

    /// Start p2p jab node
    fn start_p2p_node() -> anyhow::Result<Node> {
        let mut node = block_on(Node::init())
            .map_err(|e| anyhow::anyhow!("failed to start p2p node: {}", e))?;
        node.listen()
            .map(|_| node)
            .map_err(|e| anyhow::anyhow!("failed to start node listener: {}", e))
    }

    /// Make transaction
    fn make_transaction(
        wallet: &Wallet,
        output_address: &str,
        amount: Decimal,
    ) -> anyhow::Result<Transaction> {
        TransactionBuilder::new(TransactionVersion::V1)
            .input(wallet.address(), amount)
            .output(output_address, amount)
            .sign_with_wallet(wallet)
            .map_err(|e| anyhow::anyhow!("failed to sign transaction: {}", e))
    }

    /// Publish transaction to network and wait for response
    fn publish_transaction(
        node: &mut Node,
        transaction: Transaction,
        amount: Decimal,
        pubkey: String,
    ) -> anyhow::Result<()> {
        if let Err(err) = block_on(node.publish(Msg::transaction(
            node.id(),
            transaction.input_address().unwrap(),
            transaction.output_address().unwrap(),
            amount,
            pubkey,
            transaction.signature(),
        ))) {
            anyhow::bail!("failed to publish transaction: {}", err);
        }
        // Wait for transaction result
        match Self::wait_for_transaction_result(node) {
            TransactionResult {
                status: TransactionStatus::Ok,
                ..
            } => Ok(()),
            TransactionResult {
                error: Some(err), ..
            } => {
                anyhow::bail!("failed to publish wallet created transaction: {}", err);
            }
            res => panic!("bad message from network: {:?}", res),
        }
    }

    /// Wait for transaction result
    fn wait_for_transaction_result(node: &mut Node) -> TransactionResult {
        loop {
            match block_on(node.event_receiver.next()) {
                Some(Ok(Msg::TransactionResult(result))) => return result,
                _ => continue,
            }
        }
    }

    /// Get balance and transactions for `address`
    fn publish_get_balance(
        node: &mut Node,
        address: &str,
    ) -> anyhow::Result<(Decimal, Vec<Transaction>)> {
        if let Err(err) = block_on(node.publish(Msg::wallet_details(node.id(), address))) {
            anyhow::bail!("failed to publish wallet query: {}", err);
        }
        // Wait for transaction result
        match Self::wait_for_wallet_query_result(node) {
            WalletQueryResult::Ok(WalletTransactions {
                balance,
                transactions,
                ..
            }) => Ok((balance, transactions)),
            WalletQueryResult::Error(err) => {
                anyhow::bail!("failed to get wallet details: {}", err);
            }
        }
    }

    /// Wait for wallet query result
    fn wait_for_wallet_query_result(node: &mut Node) -> WalletQueryResult {
        loop {
            match block_on(node.event_receiver.next()) {
                Some(Ok(Msg::WalletDetailsResult(result))) => return result,
                _ => continue,
            }
        }
    }
}
