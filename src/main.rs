const JAB_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const JAB_VERSION: &str = env!("CARGO_PKG_VERSION");

// -- deps
#[macro_use]
extern crate serde;
#[macro_use]
extern crate tracing;
// -- modules
mod application;
mod blockchain;
mod mining;
mod net;

use application::Application;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("jab {} - developed by {}", JAB_VERSION, JAB_AUTHORS);
    let application = Application::init().await?;
    application.run().await
}
