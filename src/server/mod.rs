pub mod inbound;
pub mod model;
pub mod rule;

use std::sync::Arc;

use crate::common::config::Config;

use anyhow::Result;
use model::ServerConfig;

pub async fn init(config: &Config) -> Result<Arc<ServerConfig>> {
    let server_config = ServerConfig::init(config);
    let server_config = Arc::new(server_config);
    inbound::init(config, server_config.clone()).await?;
    Ok(server_config)
}
