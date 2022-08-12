//! # event
//!
//! Contains application events

use crate::net::{Msg, SwarmEvent};

/// Application event
#[derive(Debug)]
pub enum AppEvent {
    Message(Msg),
    Swarm(SwarmEvent),
    Scheduler(SchedulerEvent),
    None,
}

/// Events raised by the scheduler
#[derive(Debug)]
pub enum SchedulerEvent {
    MineBlock,
}
