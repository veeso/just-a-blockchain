//! # event
//!
//! Contains application events

use jab::net::{Msg, SwarmEvent};

/// Application event
#[derive(Debug)]
pub enum AppEvent {
    Message(Msg),
    Swarm(SwarmEvent),
    None,
}
