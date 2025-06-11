use std::sync::Arc;

use anyhow::Result;
use serde::Deserialize;
use tracing::error;

use crate::{
    protocol::{http::model::Http, socks5::model::Socks5},
    server::model::Server,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Inbound {
    Http(Http),
    Socks5(Socks5),
}

impl Inbound {
    pub fn name(&self) -> &str {
        match self {
            Inbound::Http(http) => &http.name,
            Inbound::Socks5(socks5) => &socks5.name,
        }
    }

    pub async fn listen(&self, server: Arc<Server>) -> Result<()> {
        let procotol = self.clone();
        match procotol {
            Inbound::Http(http) => {
                if let Err(e) = http.listen(server).await {
                    error!("HTTP listener error: {}", e);
                }
            }
            Inbound::Socks5(socks5) => {
                if let Err(e) = socks5.listen(server).await {
                    error!("HTTP listener error: {}", e);
                }
            }
        }
        Ok(())
    }
}
