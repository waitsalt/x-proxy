pub mod inbound;
pub mod outbound;
pub mod route;

use std::sync::OnceLock;

use anyhow::Result;
use inbound::InboundManager;
use outbound::OutboundManager;
use route::RouteManager;

use crate::config::Config;

pub static SERVICE_CONFIG: OnceLock<ServiceConfig> = OnceLock::new();

pub struct ServiceConfig {
    pub inbound_manager: InboundManager,
    pub outbound_manager: OutboundManager,
    pub route_manager: RouteManager,
}

impl ServiceConfig {
    pub fn load(config: &Config) -> Result<()> {
        let inbound = config.inbound.clone();
        let inbound_manager = InboundManager::init(inbound);
        let outbound = config.outbound.clone();
        let outbound_manager = OutboundManager::init(outbound);
        let rule_list = config.router.init();
        let global = config.router.global.clone();
        let default = config.router.default.clone();
        let route_manager = RouteManager::init(rule_list, global, default);
        if let Err(_e) = SERVICE_CONFIG.set(ServiceConfig {
            inbound_manager,
            outbound_manager,
            route_manager,
        }) {
            return Err(anyhow::anyhow!("SERVICE_CONFIG 已被初始化"));
        };
        Ok(())
    }

    pub async fn init(config: &Config) -> Result<()> {
        // 启动日志
        config.info.init();

        // 启动入站监听
        for inbound in &config.inbound {
            inbound.init().await;
        }

        Ok(())
    }
}
