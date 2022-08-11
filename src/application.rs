//! # Application
//!
//! the application module is the core of the jab client

use crate::blockchain::{Block, Chain};
use crate::net::{Msg, Node};

use tokio::time::{interval, Duration, Interval};

pub struct Application {
    blockchain: Chain,
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
        loop {
            tokio::select! {
                event = self.node.poll() => {
                    match event {
                        Ok(Some(Msg::Block(block))) => {
                            self.on_block_received(block.block).await;

                        },
                        Ok(Some(Msg::RequestBlock(block_req))) => {
                            self.on_block_requested(block_req.index).await;

                        }
                        Ok(None) => {},
                        Err(err) => {
                            error!("poll error: {}", err);
                        }
                    }
                }
                _ = self.poll_interval.tick() => {
                    self.on_tick().await;
                }
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
        info!("got a request for block #{}", requested_block);
        if let Some(block) = self.blockchain.get_block(requested_block) {
            info!("sending block #{}", requested_block);
            if let Err(err) = self.node.publish(Msg::block(block.clone())).await {
                error!("could not send `Block` message: {}", err);
            }
        } else {
            debug!("we don't currently have block #{}", requested_block);
        }
    }

    /// function to call on interval tick
    async fn on_tick(&mut self) {
        self.get_next_block().await;
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
}
