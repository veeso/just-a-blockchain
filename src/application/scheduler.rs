//! # Application scheduler
//!
//! This module expose the job scheduler for the application

use super::SchedulerEvent;

use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    stream::SelectNextSome,
    StreamExt,
};
use tokio_cron_scheduler::{Job, JobScheduler};

/// Application scheduler
pub struct Scheduler {
    scheduler: JobScheduler,
    scheduler_receiver: Option<UnboundedReceiver<SchedulerEvent>>,
}

impl Scheduler {
    /// Instantiates a new scheduler
    pub async fn new() -> anyhow::Result<Self> {
        debug!("initializing scheduler");
        let scheduler = match JobScheduler::new().await {
            Ok(s) => s,
            Err(err) => {
                anyhow::bail!("Failed to initialize job scheduler: {}", err.to_string());
            }
        };
        Ok(Self {
            scheduler,
            scheduler_receiver: None,
        })
    }

    pub fn select_next_some(&mut self) -> SelectNextSome<'_, UnboundedReceiver<SchedulerEvent>> {
        self.scheduler_receiver.as_mut().unwrap().select_next_some()
    }

    /// Setup scheduler
    pub async fn configure(&mut self) -> anyhow::Result<()> {
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
