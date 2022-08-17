//! # event
//!
//! Contains application events

use crate::net::{Msg, SwarmEvent};

/// Application event
#[derive(Debug)]
pub enum AppEvent {
    Message(Msg),
    Swarm(SwarmEvent),
    None,
}
