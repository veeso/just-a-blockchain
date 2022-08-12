//! # Application
//!
//! the application module is the core of the jab client

use crate::blockchain::{Block, Chain};
use crate::mining::{Miner, MiningDatabase};
use crate::net::{InnerSwarmEvent, Msg, Node, SwarmEvent};

use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::StreamExt;
use tokio::time::{interval, Duration, Interval};
use tokio_cron_scheduler::{Job, JobScheduler};

/// Application event
enum AppEvent {
    Message(Msg),
    Swarm(SwarmEvent),
    Scheduler(SchedulerEvent),
    None,
}

enum SchedulerEvent {
    MineBlock,
}

/// Jab client application
pub struct Application {
    blockchain: Chain,
    miners: MiningDatabase,
    node: Node,
    poll_interval: Interval,
    scheduler: JobScheduler,
    scheduler_receiver: Option<UnboundedReceiver<SchedulerEvent>>,
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
        let scheduler = match JobScheduler::new().await {
            Ok(s) => s,
            Err(err) => {
                anyhow::bail!("Failed to initialize job scheduler: {}", err.to_string());
            }
        };
        Ok(Self {
            blockchain,
            miners: MiningDatabase::new(Miner::new(node.id())),
            node,
            poll_interval: interval(Duration::from_secs(5)),
            scheduler,
            scheduler_receiver: None,
        })
    }

    /// run application
    pub async fn run(mut self) -> anyhow::Result<()> {
        if let Err(err) = self.node.listen() {
            anyhow::bail!("Failed to start listener: {}", err.to_string());
        }
        info!("listener started");
        // configure scheduler
        self.setup_scheduler().await?;
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
                event = self.scheduler_receiver.as_mut().unwrap().select_next_some() => {
                    AppEvent::Scheduler(event)
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
                AppEvent::Scheduler(event) => self.handle_scheduler_event(event).await,
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
        }
    }

    /// handle incoming event from scheduler
    async fn handle_scheduler_event(&mut self, event: SchedulerEvent) {
        match event {
            SchedulerEvent::MineBlock => {
                info!("start mining a new block");
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

    /// Setup scheduler
    async fn setup_scheduler(&mut self) -> anyhow::Result<()> {
        // setup receiver
        let (event_sender, event_receiver) = mpsc::unbounded();
        self.scheduler_receiver = Some(event_receiver);
        self.setup_mine_block_job(event_sender).await?;
        if let Err(err) = self.scheduler.start().await {
            anyhow::bail!("could not start scheduler: {}", err);
        }
        Ok(())
    }

    /// Setup mine block job
    async fn setup_mine_block_job(
        &mut self,
        event_sender: UnboundedSender<SchedulerEvent>,
    ) -> anyhow::Result<()> {
        // mine block job
        let mining_job = match Job::new("30 * * * * *", move |_uuid, _lock| {
            if let Err(err) = event_sender.unbounded_send(SchedulerEvent::MineBlock) {
                error!("failed to send to receiver (thread): {}", err);
            }
        }) {
            Ok(j) => j,
            Err(err) => {
                anyhow::bail!("could not create MineBlock job: {}", err);
            }
        };
        if let Err(err) = self.scheduler.add(mining_job).await {
            anyhow::bail!("could not schedule MineBlock job: {}", err);
        }
        Ok(())
    }
}
