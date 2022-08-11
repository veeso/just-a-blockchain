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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("jab {} - developed by {}", JAB_VERSION, JAB_AUTHORS);

    let mut node = match net::Node::init().await {
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

    // loop
    loop {
        tokio::select! {
            _ = node.poll() => {}
        }
    }
}
