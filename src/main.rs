use std::sync::Arc;

use anyhow::Result;
use x_proxy::{common::config::model::Config, server::model::Server};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;
    config.init()?;

    let server = Server::init(&config);
    let server = Arc::new(server);

    server.start(&config, server.clone()).await?;

    tokio::signal::ctrl_c().await?;

    Ok(())
}
