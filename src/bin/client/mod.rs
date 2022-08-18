//! # Client application
//!
//! This module exposes the main client application

use std::path::Path;

use crate::Args;

use futures::StreamExt;
use jab::blockchain::{Chain, Transaction, TransactionBuilder, TransactionVersion};
use jab::net::{
    message::{TransactionResult, TransactionStatus, WalletQueryResult, WalletTransactions},
    Msg, Node,
};
use jab::wallet::{Wallet, SECRET_KEY_SIZE};
pub use libp2p::swarm::SwarmEvent;
use merkle::Hashable;
use ring::digest::{Context, SHA256};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fs;
use std::io::{Read, Write};
use std::str::FromStr;
use tracing::debug;

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
    pub async fn run(task: Task, args: Args) -> anyhow::Result<()> {
        match task {
            Task::GenerateNewWallet => Self::generate_new_wallet(&args.wallet).await,
            Task::GetBalance => Self::get_balance(&args.wallet).await,
            Task::GetBalanceFor(addr) => Self::get_balance_for(&addr).await,
            Task::Send => Self::send(&args.wallet).await,
            Task::SignGenesisBlock => Self::sign_genesis_block(&args.wallet),
            Task::None => Ok(()),
        }
    }

    /// generate new wallet for client
    async fn generate_new_wallet(p: &Path) -> anyhow::Result<()> {
        let wallet = Wallet::new();
        debug!("generated new wallet with address {}", wallet.address());
        // create directory
        if let Err(err) = fs::create_dir_all(p) {
            anyhow::bail!("could not create directory at {}: {}", p.display(), err);
        }
        debug!("created wallet directories");
        // write keys
        Self::write_key(p, WALLET_PUBLIC_KEY, wallet.public_key().as_bytes())?;
        Self::write_key(p, WALLET_SECRET_KEY, &wallet.secret_key())?;
        debug!("written keys to {}", p.display());
        // publish wallet to blockchain
        let transaction = Self::make_transaction(&wallet, wallet.address(), Decimal::ZERO)?;
        debug!("prepared wallet registration transaction");
        let mut node = Self::start_p2p_node().await?;
        Self::publish_transaction(&mut node, transaction, Decimal::ZERO, wallet.public_key())
            .await?;
        println!("created new wallet at {}", p.display());
        println!("your address is: {}", wallet.address());
        Ok(())
    }

    /// Get balance for this wallet
    async fn get_balance(p: &Path) -> anyhow::Result<()> {
        let wallet = Self::open_wallet(p)?;
        Self::get_balance_for(wallet.address()).await
    }

    /// Get balance for provided address
    async fn get_balance_for(address: &str) -> anyhow::Result<()> {
        debug!("getting balance for {}", address);
        let mut node = Self::start_p2p_node().await?;
        let (balance, transactions) = Self::publish_get_balance(&mut node, address).await?;
        for transaction in transactions.into_iter() {
            for input in transaction
                .inputs()
                .iter()
                .filter(|x| x.address.as_str() == address)
            {
                println!("SPENT {} JAB", input.amount);
            }
            for output in transaction
                .outputs()
                .iter()
                .filter(|x| x.address.as_str() == address)
            {
                println!("RECEIVED {} JAB", output.amount);
            }
        }
        println!("wallet amount for {}: {}", address, balance);
        Ok(())
    }

    /// Send money from this wallet to another
    async fn send(p: &Path) -> anyhow::Result<()> {
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
            Decimal::from_str(amount.trim()).map_err(|e| anyhow::anyhow!("bad amount: {}", e))?;
        debug!("sending {} to {}", amount, recipient);
        // send
        let transaction = Self::make_transaction(&wallet, recipient.trim(), amount)?;
        let mut node = Self::start_p2p_node().await?;
        Self::publish_transaction(&mut node, transaction, amount, wallet.public_key()).await?;
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
    async fn start_p2p_node() -> anyhow::Result<Node> {
        debug!("starting p2p node");
        let mut node = Node::init()
            .await
            .map_err(|e| anyhow::anyhow!("failed to start p2p node: {}", e))?;
        debug!("starting p2p listener");
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
    async fn publish_transaction(
        node: &mut Node,
        transaction: Transaction,
        amount: Decimal,
        pubkey: String,
    ) -> anyhow::Result<()> {
        debug!("publishing transaction {:?}", transaction);
        // Wait for transaction result
        match Self::wait_for_transaction_result(
            node,
            Msg::transaction(
                node.id(),
                transaction.input_address().unwrap(),
                transaction.output_address().unwrap(),
                amount,
                pubkey,
                transaction.signature(),
            ),
        )
        .await
        {
            Ok(TransactionResult {
                status: TransactionStatus::Ok,
                ..
            }) => Ok(()),
            Ok(TransactionResult {
                error: Some(err), ..
            }) => {
                anyhow::bail!("failed to publish wallet created transaction: {}", err);
            }
            Err(err) => Err(err),
            res => panic!("bad message from network: {:?}", res),
        }
    }

    /// Wait for transaction result
    async fn wait_for_transaction_result(
        node: &mut Node,
        msg: Msg,
    ) -> anyhow::Result<TransactionResult> {
        let mut should_publish_transaction = false;
        loop {
            let event = tokio::select! {
                message = node.swarm.select_next_some() => {
                    if matches!(message, SwarmEvent::ConnectionEstablished { .. } | SwarmEvent::ConnectionClosed { .. }) {
                        should_publish_transaction = true;
                    }
                    None
                },
                message = node.event_receiver.next() => {
                    match message {
                        Some(Ok(Msg::TransactionResult(result))) => Some(result),
                        _ => None,
                    }
                }
            };
            if should_publish_transaction {
                if let Err(err) = node.publish(msg.clone()).await {
                    anyhow::bail!("failed to publish transaction: {}", err);
                } else {
                    should_publish_transaction = false;
                }
            }
            if let Some(event) = event {
                return Ok(event);
            }
        }
    }

    /// Get balance and transactions for `address`
    async fn publish_get_balance(
        node: &mut Node,
        address: &str,
    ) -> anyhow::Result<(Decimal, Vec<Transaction>)> {
        debug!("publishing wallet details query for {}", address);
        // Wait for transaction result
        match Self::wait_for_wallet_query_result(node, Msg::wallet_details(node.id(), address))
            .await
        {
            Ok(WalletQueryResult::Ok(WalletTransactions {
                balance,
                transactions,
                ..
            })) => Ok((balance, transactions)),
            Ok(WalletQueryResult::Error(err)) => {
                anyhow::bail!("failed to get wallet details: {}", err);
            }
            Err(err) => Err(err),
        }
    }

    /// Wait for wallet query result
    async fn wait_for_wallet_query_result(
        node: &mut Node,
        msg: Msg,
    ) -> anyhow::Result<WalletQueryResult> {
        let mut should_publish_transaction = false;
        loop {
            let event = tokio::select! {
                message = node.swarm.select_next_some() => {
                    if matches!(message, SwarmEvent::ConnectionEstablished { .. } | SwarmEvent::ConnectionClosed { .. }) {
                        should_publish_transaction = true;
                    }
                    None
                },
                message = node.event_receiver.next() => {
                    match message {
                        Some(Ok(Msg::WalletDetailsResult(result))) => Some(result),
                        _ => None,
                    }
                }
            };
            if should_publish_transaction {
                if let Err(err) = node.publish(msg.clone()).await {
                    anyhow::bail!("failed to publish wallet query: {}", err);
                } else {
                    should_publish_transaction = false;
                }
            }
            if let Some(event) = event {
                return Ok(event);
            }
        }
    }
}
