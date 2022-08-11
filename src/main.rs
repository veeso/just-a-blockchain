const JAB_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const JAB_VERSION: &str = env!("CARGO_PKG_VERSION");

// -- deps
#[macro_use]
extern crate serde;
#[macro_use]
extern crate tracing;
// -- modules
mod blockchain;
mod net;

use blockchain::Chain;
use net::{Msg, Node};

use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("jab {} - developed by {}", JAB_VERSION, JAB_AUTHORS);
    // setup blockchain
    let mut blockchain = Chain::new();
    info!("blockchain ready!");
    // setup node
    let mut node = match Node::init().await {
        Ok(node) => node,
        Err(err) => {
            anyhow::bail!("Failed to initialize node: {}", err.to_string());
        }
    };
    info!("node successfully initialized (id: {})", node.id());
    // listen
    if let Err(err) = node.listen() {
        anyhow::bail!("Failed to start listener: {}", err.to_string());
    }
    info!("listener started");
    // latest block poll interval
    let mut poll_interval = interval(Duration::from_secs(5));

    // loop
    loop {
        tokio::select! {
            event = node.poll() => {
                match event {
                    Ok(Some(Msg::Block(block))) => {
                        let block = block.block;
                        let block_index = block.index();
                        info!("received block #{}", block_index);
                        if let Err(err) = blockchain.add_block(block) {
                            error!("could not add block #{}: {}", block_index, err);
                        }
                        // TODO: request next
                    },
                    Ok(Some(Msg::RequestBlock(block_req))) => {
                        let requested_block = block_req.index;
                        info!("requested block #{}", requested_block);
                        if let Some(block) = blockchain.get_block(requested_block) {
                            info!("sending block #{}", requested_block);
                            if let Err(err) = node.publish(Msg::block(block.clone())).await {
                                error!("could not send `Block` message: {}", err);
                            }
                        } else {
                            debug!("we don't currently have block #{}", requested_block);
                        }
                    }
                    Ok(None) => {},
                    Err(err) => {
                        error!("poll error: {}", err);
                    }
                }
            }
            _ = poll_interval.tick() => {
                let next_index = blockchain.get_latest_block().index() + 1;
                match node.publish(Msg::request_block(next_index)).await {
                    Ok(()) => {
                        debug!("requested block #{}", next_index);
                    }
                    Err(err) => {
                        error!("failed to request block #{}: {}", next_index, err);
                    }
                }
            }
        }
    }
}
