//! # Net
//!
//! The network module provides the types to setup the P2P network of the jab blockchain

mod message;

use futures::StreamExt;
use libp2p::{
    core::upgrade,
    floodsub::{self, Floodsub, FloodsubEvent, FloodsubMessage, Topic},
    identity::{self, Keypair},
    mplex,
    noise::{self, NoiseError},
    swarm::{Swarm, SwarmBuilder, SwarmEvent},
    tcp::TokioTcpTransport,
    NetworkBehaviour, PeerId, Transport, TransportError,
};
use libp2p_tcp::GenTcpConfig;
use thiserror::Error;

pub use message::Msg;

/// Node result
pub type NodeResult<T> = Result<T, NodeError>;

/// Node error
#[derive(Error, Debug)]
pub enum NodeError {
    #[error("io error: {0}")]
    Io(std::io::Error),
    #[error("invalid payload codec: {0}")]
    InvalidPayload(serde_json::Error),
    #[error("noise error: {0}")]
    Noise(NoiseError),
    #[error("transport error: {0}")]
    TransportError(TransportError<std::io::Error>),
}

impl From<serde_json::Error> for NodeError {
    fn from(e: serde_json::Error) -> Self {
        Self::InvalidPayload(e)
    }
}

impl From<NoiseError> for NodeError {
    fn from(e: NoiseError) -> Self {
        Self::Noise(e)
    }
}

impl From<std::io::Error> for NodeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<TransportError<std::io::Error>> for NodeError {
    fn from(e: TransportError<std::io::Error>) -> Self {
        Self::TransportError(e)
    }
}

/// Represents the client node in the p2p network
#[allow(dead_code)]
pub struct Node {
    id: PeerId,
    keys: Keypair,
    swarm: Swarm<JabBehaviour>,
    topic: Topic,
}

impl Node {
    /// Initialize a new `Node`
    pub async fn init() -> NodeResult<Self> {
        // generate keys
        let id_keys = identity::Keypair::generate_ed25519();
        let id = PeerId::from(id_keys.public());
        info!("initializing new Node with id: {}", id);
        // Create a keypair for authenticated encryption of the transport.
        let noise_keys = noise::Keypair::<noise::X25519Spec>::new().into_authentic(&id_keys)?;
        debug!("generated noise keys");
        // Create a tokio-based TCP transport use noise for authenticated
        // encryption and Mplex for multiplexing of substreams on a TCP stream.
        let transport = TokioTcpTransport::new(GenTcpConfig::default().nodelay(true))
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
            .multiplex(mplex::MplexConfig::new())
            .boxed();
        debug!("tcp transport setup ok");
        // setup topic
        let topic = floodsub::Topic::new("jab");

        // Create a Swarm to manage peers and events.
        let swarm = {
            let mut behaviour = JabBehaviour {
                floodsub: Floodsub::new(id),
            };

            behaviour.floodsub.subscribe(topic.clone());
            // setup swarm
            SwarmBuilder::new(transport, behaviour, id)
                // We want the connection background tasks to be spawned
                // onto the tokio runtime.
                .executor(Box::new(|fut| {
                    tokio::spawn(fut);
                }))
                .build()
        };
        Ok(Node {
            id,
            keys: id_keys,
            swarm,
            topic,
        })
    }

    /// Get peer id as string
    pub fn id(&self) -> String {
        self.id.to_string()
    }

    /// Start listener on a random OS port
    pub fn listen(&mut self) -> NodeResult<()> {
        self.swarm
            .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
            .map(|_| ())
            .map_err(NodeError::from)
    }

    /// Poll for incoming messages
    pub async fn poll(&mut self) -> NodeResult<Option<Msg>> {
        Ok(match self.swarm.select_next_some().await {
            SwarmEvent::Behaviour(OutEvent::Floodsub(FloodsubEvent::Message(
                FloodsubMessage { source, data, .. },
            ))) => {
                debug!("received message from {}", source);
                // decode message
                Some(serde_json::from_slice(&data).map_err(NodeError::from)?)
            }
            _ => None,
        })
    }

    /// Publish a message to the newtwork
    pub async fn publish(&mut self, message: Msg) -> NodeResult<()> {
        debug!("publishing {:?}", message);
        self.swarm.behaviour_mut().floodsub.publish(
            self.topic.clone(),
            serde_json::json!(message).to_string().as_bytes(),
        );
        Ok(())
    }
}

// We create a custom network behaviour that combines floodsub and mDNS.
// The derive generates a delegating `NetworkBehaviour` impl which in turn
// requires the implementations of `NetworkBehaviourEventProcess` for
// the events of each behaviour.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent")]
struct JabBehaviour {
    floodsub: Floodsub,
}

#[derive(Debug)]
enum OutEvent {
    Floodsub(FloodsubEvent),
}

impl From<FloodsubEvent> for OutEvent {
    fn from(v: FloodsubEvent) -> Self {
        Self::Floodsub(v)
    }
}
