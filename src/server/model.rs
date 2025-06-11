use std::sync::Arc;

use crate::{
    common::config::Config, inbound::model::InboundManager, outbound::model::OutboundManager,
    route::model::RouteManager,
};

pub struct ServerConfig {
    pub inbound_manager: Arc<InboundManager>,
    pub outbound_manager: Arc<OutboundManager>,
    pub route_manager: Arc<RouteManager>,
}

impl ServerConfig {
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
}
