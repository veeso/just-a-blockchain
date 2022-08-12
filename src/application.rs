//! # Application
//!
//! the application module is the core of the jab client

use crate::blockchain::{Block, Chain};
use crate::mining::{Miner, MiningDatabase};
use crate::net::{Msg, Node};

use futures::StreamExt;
use tokio::time::{interval, Duration, Interval};

/// Jab client application
pub struct Application {
    blockchain: Chain,
    miners: MiningDatabase,
    node: Node,
    poll_interval: Interval,
}

impl Application {
    /// Initialize new `Application`
    pub async fn init() -> anyhow::Result<Self> {
        // setup blockchain
        let blockchain = Chain::new();
        info!("blockchain ready!");
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
            let message: Option<Msg> = tokio::select! {
                event = self.node.swarm.next() => {
                    debug!("Unhandled Swarm Event: {:?}", event);
                    None
                }
                message = self.node.event_receiver.next() => {
                    match message {
                        Some(Ok(message)) => Some(message),
                        _ => None,
                    }
                }
                _ = self.poll_interval.tick() => {
                    self.on_get_next_block_tick().await;
                    self.send_miner_requests().await;
                    self.poll_interval.reset();
                    None
                }
            };
            if let Some(message) = message {
                self.handle_message(message).await;
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
        }
    }

    /// code to run on block received
    async fn on_block_received(&mut self, block: Block) {
        let block_index = block.index();
        info!("received block #{}", block_index);
        if let Err(err) = self.blockchain.add_block(block) {
            error!("could not add block #{}: {}", block_index, err);
        }
        // request next block
        self.get_next_block().await;
    }

    /// code to run on block requested
    async fn on_block_requested(&mut self, requested_block: u64) {
        debug!("got a request for block #{}", requested_block);
        if let Some(block) = self.blockchain.get_block(requested_block) {
            debug!("sending block #{}", requested_block);
            if let Err(err) = self.node.publish(Msg::block(block.clone())).await {
                error!("could not send `Block` message: {}", err);
            }
        } else {
            debug!("we don't currently have block #{}", requested_block);
        }
    }

    /// Function to execute on a `RegisterMiners` message
    async fn on_register_miners(&mut self, miners: Vec<Miner>) {
        debug!("received new miners database");
        for miner in miners.into_iter() {
            self.miners.add_miner(miner);
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

    /// function to execute after the miner_db_timeout elapsed
    async fn send_miner_requests(&mut self) {
        // send current miner database
        self.send_miners_database().await;
        // request m iners database
        self.request_registered_miners().await;
    }

    /// get next block from other peer through a request
    async fn get_next_block(&mut self) {
        let next_index = self.blockchain.get_latest_block().index() + 1;
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
}
