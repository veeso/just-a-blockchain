const JAB_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const JAB_VERSION: &str = env!("CARGO_PKG_VERSION");

// -- deps
#[macro_use]
extern crate serde;
#[macro_use]
extern crate tracing;
// -- modules
mod application;

use application::{Application, Config as AppConfig};
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    info!("jab {} - developed by {}", JAB_VERSION, JAB_AUTHORS);
    let config = AppConfig::try_from_env()?;
    info!("configuration successfully loaded");
    let application = Application::init(config).await?;
    info!("application ready!");
    application.run().await
}
