//! # Application
//!
//! the application module is the core of the jab client

// -- modules
mod config;
mod event;
mod transaction_helper;
mod wallet_helper;

pub use config::Config;
use rust_decimal::Decimal;

use event::AppEvent;
use jab::blockchain::{Block, Chain, Transaction};
use jab::mining::{Miner, MiningDatabase};
use jab::net::{
    message::{Transaction as MsgTransaction, WalletQuery, WalletQueryError},
    InnerSwarmEvent, Msg, Node, SwarmEvent,
};
use jab::wallet::Wallet;
use transaction_helper::{TransactionHelper, TransactionOptions, TransactionRejected};
use wallet_helper::WalletHelper;

use futures::StreamExt;
use tokio::time::{interval, Duration, Interval};

/// Jab client application
pub struct Application {
    blockchain: Chain,
    miners: MiningDatabase,
    node: Node,
    poll_interval: Interval,
    wallet: Wallet,
}

impl Application {
    /// Initialize new `Application`
    pub async fn init(config: Config) -> anyhow::Result<Self> {
        // setup blockchain
        let blockchain = Chain::try_from(config.database_dir())?;
        info!(
            "blockchain ready! Found {} blocks",
            blockchain.get_latest_block()?.index()
        );
        // setup node
        let node = match Node::init().await {
            Ok(node) => node,
            Err(err) => {
                anyhow::bail!("Failed to initialize node: {}", err.to_string());
            }
        };
        info!("node successfully initialized (id: {})", node.id());
        Ok(Self {
            blockchain,
            miners: MiningDatabase::new(Miner::new(node.id())),
            node,
            poll_interval: interval(Duration::from_secs(5)),
            wallet: WalletHelper::open_wallet(config.wallet_secret_key()).await?,
        })
    }

    /// run application
    pub async fn run(mut self) -> anyhow::Result<()> {
        if let Err(err) = self.node.listen() {
            anyhow::bail!("Failed to start listener: {}", err.to_string());
        }
        info!("listener started");
        // main loop
        loop {
            let event: AppEvent = tokio::select! {
                event = self.node.swarm.select_next_some() => AppEvent::Swarm(event),
                message = self.node.event_receiver.next() => {
                    match message {
                        Some(Ok(message)) => AppEvent::Message(message),
                        _ => AppEvent::None,
                    }
                }
                _ = self.poll_interval.tick() => {
                    self.on_get_next_block_tick().await;
                    // if currently there's only one known miner (which is us), send requests for discovering miners
                    if self.miners.miners().len() == 1 {
                        self.send_miner_requests().await;
                    }
                    self.poll_interval.reset();
                    AppEvent::None
                }
            };
            match event {
                AppEvent::Message(message) => self.handle_message(message).await,
                AppEvent::Swarm(event) => self.handle_swarm_event(event).await,
                AppEvent::None => {}
            }
        }
    }

    /// handle incoming message from peer
    async fn handle_message(&mut self, message: Msg) {
        match message {
            Msg::Block(block) => {
                self.on_block_received(block.block).await;
            }
            Msg::RequestBlock(block_req) => {
                self.on_block_requested(block_req.index).await;
            }
            Msg::RegisterMiners(miners) => {
                self.on_register_miners(miners.miners).await;
            }
            Msg::RequestRegisteredMiners => {
                self.on_registered_miners_requested().await;
            }
            Msg::Transaction(transaction) => {
                self.on_transaction(transaction).await;
            }
            Msg::TransactionResult(_) => {
                debug!("ignoring transaction result");
            }
            Msg::WalletDetails(query) => {
                self.on_wallet_details_query(query).await;
            }
            Msg::WalletDetailsResult(_) => {
                debug!("ignoring wallet details result");
            }
        }
    }

    /// handle incoming event from swarm
    async fn handle_swarm_event(&mut self, event: SwarmEvent) {
        match event {
            InnerSwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!(
                    "connection closed with {}; unregistering peer from miners",
                    peer_id
                );
                self.miners.unregister_miner(peer_id);
            }
            _ => {
                debug!("unhandled swarm event: {:?}", event);
            }
        }
    }

    /// code to run on block received
    async fn on_block_received(&mut self, block: Block) {
        let block_index = block.index();
        info!(
            "received block #{} with hash {}",
            block_index,
            block.header().merkle_root_hash()
        );
        if let Err(err) = self.blockchain.add_block(block) {
            error!("could not add block #{}: {}", block_index, err);
        }
        // request next block
        self.get_next_block().await;
    }

    /// code to run on block requested
    async fn on_block_requested(&mut self, requested_block: u64) {
        debug!("got a request for block #{}", requested_block);
        match self.blockchain.get_block(requested_block) {
            Err(err) => {
                error!(
                    "can't retrieve block #{} from database: {}",
                    requested_block, err
                );
            }
            Ok(None) => {
                debug!("we don't currently have block #{}", requested_block);
            }
            Ok(Some(block)) => {
                debug!("sending block #{}", requested_block);
                if let Err(err) = self.node.publish(Msg::block(block.clone())).await {
                    error!("could not send `Block` message: {}", err);
                }
            }
        }
    }

    /// Function to execute on a `RegisterMiners` message
    async fn on_register_miners(&mut self, miners: Vec<Miner>) {
        debug!("received new miners database");
        for miner in miners.into_iter() {
            self.miners.register_miner(miner);
        }
    }

    /// Function to execute on a `RequestRegisteredMiners` message
    async fn on_registered_miners_requested(&mut self) {
        debug!("received a request for registered miners; sending database");
        self.send_miners_database().await;
    }

    /// function to call on interval tick
    async fn on_get_next_block_tick(&mut self) {
        self.get_next_block().await;
    }

    /// Function to handle a `WalletDetails` query
    async fn on_wallet_details_query(&mut self, query: WalletQuery) {
        debug!("received wallet query for {}", query.address);
        // collect transactions
        match self.blockchain.wallet_transactions(&query.address) {
            Err(_) => {
                self.send_wallet_details_error(&query.peer_id, WalletQueryError::BlockchainError)
                    .await
            }
            Ok(None) => {
                self.send_wallet_details_error(&query.peer_id, WalletQueryError::WalletNotFound)
                    .await
            }
            Ok(Some(transactions)) => {
                // calc wallet balance
                let mut balance = Decimal::ZERO;
                for transaction in transactions.iter() {
                    balance -= transaction.amount_spent(&query.address);
                    balance += transaction.amount_received(&query.address);
                }
                debug!(
                    "found {} transactions for wallet {}; current amount {} JAB",
                    transactions.len(),
                    query.address,
                    balance
                );
                self.send_wallet_details_ok(&query.peer_id, &query.address, transactions, balance)
                    .await
            }
        }
    }

    /// function to execute after the miner_db_timeout elapsed
    async fn send_miner_requests(&mut self) {
        // send current miner database
        self.send_miners_database().await;
        // request m iners database
        self.request_registered_miners().await;
    }

    /// get next block from other peer through a request
    async fn get_next_block(&mut self) {
        let next_index = match self.blockchain.get_latest_block() {
            Ok(block) => block.index() + 1,
            Err(err) => {
                error!("could not get the latest block: {}", err);
                return;
            }
        };
        match self.node.publish(Msg::request_block(next_index)).await {
            Ok(()) => {
                debug!("requested block #{}", next_index);
            }
            Err(err) => {
                error!("failed to request block #{}: {}", next_index, err);
            }
        }
    }

    /// Send miners database
    async fn send_miners_database(&mut self) {
        debug!("sending miners database");
        if let Err(err) = self
            .node
            .publish(Msg::register_miners(self.miners.miners()))
            .await
        {
            error!("failed to send registered miners: {}", err);
        }
    }

    /// Send a request for the registered miners database
    async fn request_registered_miners(&mut self) {
        debug!("sending registered miners request");
        if let Err(err) = self.node.publish(Msg::request_registered_miners()).await {
            error!("failed to request registered miners: {}", err);
        }
    }

    /// `Transaction` message handler.
    /// It tries to register the transaction in the blockchain and send a response to the requesting peer
    async fn on_transaction(&mut self, transaction_msg: MsgTransaction) {
        info!(
            "requested transaction from {} to {}; amount: {}",
            transaction_msg.input_address, transaction_msg.output_address, transaction_msg.amount
        );
        // Make transaction
        let transaction = match TransactionHelper::create_transaction(
            TransactionOptions::new(
                transaction_msg.input_address,
                transaction_msg.output_address,
            )
            .amount(transaction_msg.amount)
            .fee(rust_decimal_macros::dec!(0.02))
            .signature(transaction_msg.signature)
            .public_key(transaction_msg.public_key),
            &self.wallet,
            &self.blockchain,
        )
        .await
        {
            Ok(t) => t,
            Err(e) => {
                self.send_transaction_response_nok(&transaction_msg.peer_id, e)
                    .await;
                return;
            }
        };
        // generate next block
        self.miners.set_last_block_miner();
        let new_block = match self.blockchain.generate_next_block(transaction) {
            Ok(block) => block,
            Err(err) => {
                error!("could not generate new block: {}", err);
                self.send_transaction_response_nok(
                    &transaction_msg.peer_id,
                    TransactionRejected::BlockchainError(err),
                )
                .await;
                return;
            }
        };
        info!(
            "generated block #{}, with hash {}",
            new_block.index(),
            new_block.header().merkle_root_hash()
        );
        // send response OK
        self.send_transaction_response_ok(&transaction_msg.peer_id)
            .await;
        // send new block to other peers
        if let Err(err) = self.node.publish(Msg::block(new_block.clone())).await {
            error!("failed to send new block to peers: {}", err);
        }
        info!(
            "block #{} successfully broadcasted to peer",
            new_block.index()
        );
    }

    /// Send transaction response NOK to peer
    async fn send_transaction_response_nok(&mut self, peer_id: &str, error: TransactionRejected) {
        debug!("sending transaction response NOK to {}", peer_id);
        let description = error.to_string();
        if let Err(err) = self
            .node
            .send(
                peer_id,
                Msg::transaction_result_nok(error.into(), description),
            )
            .await
        {
            error!(
                "could not send transaction response to {}: {}",
                peer_id, err
            );
        }
    }

    /// Send transaction response OK to peer
    async fn send_transaction_response_ok(&mut self, peer_id: &str) {
        debug!("sending transaction response OK to {}", peer_id);
        if let Err(err) = self.node.send(peer_id, Msg::transaction_result_ok()).await {
            error!(
                "could not send transaction response to {}: {}",
                peer_id, err
            );
        }
    }

    /// Send wallet details response ERROR to peer
    async fn send_wallet_details_error(&mut self, peer_id: &str, error: WalletQueryError) {
        debug!(
            "sending wallet details response ERROR to {}: {}",
            peer_id, error
        );
        if let Err(err) = self
            .node
            .send(peer_id, Msg::wallet_details_result_error(error))
            .await
        {
            error!(
                "could not send wallet details response to {}: {}",
                peer_id, err
            );
        }
    }

    /// Send wallet details response OK to peer
    async fn send_wallet_details_ok(
        &mut self,
        peer_id: &str,
        address: &str,
        transactions: Vec<Transaction>,
        balance: Decimal,
    ) {
        debug!("sending wallet details response OK to {}", peer_id);
        if let Err(err) = self
            .node
            .send(
                peer_id,
                Msg::wallet_details_result_ok(address, transactions, balance),
            )
            .await
        {
            error!(
                "could not send wallet details response to {}: {}",
                peer_id, err
            );
        }
    }
}
