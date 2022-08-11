//! # Net
//!
//! The network module provides the types to setup the P2P network of the jab blockchain

mod error;
mod message;

use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use libp2p::{
    core::upgrade,
    floodsub::{self, Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::{Mdns, MdnsEvent},
    mplex, noise,
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder},
    tcp::TokioTcpTransport,
    NetworkBehaviour, PeerId, Transport,
};
use libp2p_tcp::GenTcpConfig;

pub use error::{NodeError, NodeResult};
pub use message::Msg;

/// Represents the client node in the p2p network
pub struct Node {
    id: PeerId,
    pub swarm: Swarm<JabBehaviour>,
    topic: Topic,
    pub event_receiver: UnboundedReceiver<NodeResult<Msg>>,
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
            let mut behaviour = JabBehaviour {
                floodsub: Floodsub::new(id),
                mdns: Mdns::new(Default::default()).await?,
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
#[behaviour(event_process = true)]
pub struct JabBehaviour {
    floodsub: Floodsub,
    mdns: Mdns,
    #[behaviour(ignore)]
    event_sender: UnboundedSender<NodeResult<Msg>>,
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for JabBehaviour {
    // Called when `floodsub` produces an event.
    fn inject_event(&mut self, message: FloodsubEvent) {
        if let FloodsubEvent::Message(message) = message {
            debug!(
                "Received: message from {} {}",
                message.source,
                String::from_utf8_lossy(&message.data)
            );
            // decode message
            let ev_sender = self.event_sender.clone();
            let message = serde_json::from_slice(&message.data).map_err(NodeError::from);
            tokio::spawn(async move {
                if let Err(err) = ev_sender.unbounded_send(message) {
                    error!("failed to send to receiver (thread): {}", err);
                }
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
