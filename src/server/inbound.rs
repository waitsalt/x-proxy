use super::model::ServerConfig;
use crate::{common::config::Config, inbound::model::Inbound};
use anyhow::Result;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::error;

pub async fn init(config: &Config, server_config: Arc<ServerConfig>) -> Result<()> {
    let inbound_list = config.inbound.clone();
    let mut handles = Vec::new();

    for inbound in inbound_list {
        let server_config = server_config.clone();

        let handle: JoinHandle<()> = match inbound {
            Inbound::Http(http) => tokio::spawn(async move {
                if let Err(e) = http.inbound(server_config).await {
                    error!("HTTP inbound error: {}", e);
                }
            }),
            Inbound::Socks5(socks5) => tokio::spawn(async move {
                if let Err(e) = socks5.inbound(server_config).await {
                    error!("SOCKS5 inbound error: {}", e);
                }
            }),
        };

        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}
