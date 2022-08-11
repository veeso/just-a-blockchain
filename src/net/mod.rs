//! # Net
//!
//! The network module provides the types to setup the P2P network of the jab blockchain

mod message;

use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::{SinkExt, StreamExt};
use libp2p::{
    core::upgrade,
    floodsub::{self, Floodsub, FloodsubEvent, Topic},
    identity::{self, Keypair},
    mdns::{Mdns, MdnsEvent},
    mplex,
    noise::{self, NoiseError},
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder},
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
    event_receiver: UnboundedReceiver<NodeResult<Msg>>,
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
        let (event_sender, event_receiver) = mpsc::unbounded();
        // Create a Swarm to manage peers and events.
        let swarm = {
            let mdns = Mdns::new(Default::default()).await?;
            let mut behaviour = JabBehaviour {
                floodsub: Floodsub::new(id),
                mdns,
                event_sender,
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
            event_receiver,
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
        let _ = self.swarm.select_next_some();
        match self.event_receiver.try_next() {
            Ok(Some(result)) => result.map(Some),
            Ok(None) | Err(_) => Ok(None),
        }
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
#[behaviour(out_event = "OutEvent", event_process = true)]
struct JabBehaviour {
    floodsub: Floodsub,
    mdns: Mdns,
    #[behaviour(ignore)]
    event_sender: UnboundedSender<NodeResult<Msg>>,
}

#[derive(Debug)]
enum OutEvent {
    Floodsub(FloodsubEvent),
    Mdns(MdnsEvent),
}

impl From<FloodsubEvent> for OutEvent {
    fn from(v: FloodsubEvent) -> Self {
        Self::Floodsub(v)
    }
}

impl From<MdnsEvent> for OutEvent {
    fn from(v: MdnsEvent) -> Self {
        Self::Mdns(v)
    }
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for JabBehaviour {
    // Called when `floodsub` produces an event.
    fn inject_event(&mut self, message: FloodsubEvent) {
        if let FloodsubEvent::Message(message) = message {
            debug!("Received: message from {}", message.source);
            // decode message
            let mut ev_sender = self.event_sender.clone();
            let message = serde_json::from_slice(&message.data).map_err(NodeError::from);
            tokio::spawn(async move {
                let _ = ev_sender.send(message).await;
            });
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for JabBehaviour {
    // Called when `mdns` produces an event.
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}
