use std::sync::Arc;

use anyhow::Result;
use tracing::error;

use crate::common::config::model::Config;

use super::{inbound::InboundManager, outbound::OutboundManager, route::RouteManager};

pub struct Server {
    pub inbound_manager: Arc<InboundManager>,
    pub outbound_manager: Arc<OutboundManager>,
    pub route_manager: Arc<RouteManager>,
}

impl Server {
    pub fn init(config: &Config) -> Self {
        let inbound = config.inbound.clone();
        let inbound_manager = Arc::new(InboundManager::init(inbound));
        let outbound = config.outbound.clone();
        let outbound_manager = Arc::new(OutboundManager::init(outbound));
        let rule_list = config.route.init();
        let default = config.route.default.clone();
        let route_manager = Arc::new(RouteManager::init(rule_list, default));

        Self {
            inbound_manager,
            outbound_manager,
            route_manager,
        }
    }

    pub async fn start(&self, config: &Config, server: Arc<Server>) -> Result<()> {
        let inbound_vec = config.inbound.clone();
        for inboud in inbound_vec {
            let server = server.clone();
            tokio::spawn(async move {
                if let Err(e) = inboud.listen(server).await {
                    error!("inbound {} start error: {}", inboud.name(), e);
                }
            });
        }

        Ok(())
    }
}
