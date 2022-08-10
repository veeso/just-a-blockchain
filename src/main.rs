const JAB_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const JAB_VERSION: &str = env!("CARGO_PKG_VERSION");

// -- deps
#[macro_use]
extern crate serde;
#[macro_use]
extern crate tracing;
// -- modules
mod net;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("jab {} - developed by {}", JAB_VERSION, JAB_AUTHORS);

    let mut node = net::Node::init().await.expect("could not init node");
    info!("node successfully initialized");
    // listen
    node.listen().expect("failed to start listener");

    // loop
    loop {
        tokio::select! {
            _ = node.poll() => {}
        }
    }
}
