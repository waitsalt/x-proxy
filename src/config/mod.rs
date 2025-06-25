pub mod inbound;
pub mod info;
pub mod mode;
pub mod outbound;
pub mod router;

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use tracing::warn;

use inbound::Inbound;
use info::Info;
use mode::Mode;
use outbound::Outbound;
use router::Router;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub info: Info,
    pub mode: Mode,
    pub router: Router,
    pub inbound: Vec<Inbound>,
    pub outbound: Vec<Outbound>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let file_path = "config.yaml";
        let config_str = std::fs::read_to_string(file_path)
            .with_context(|| format!("读取配置文件失败: {}", file_path))?;
        let config: Config = serde_yaml::from_str(&config_str).with_context(|| "解析文件失败")?;

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // 验证入站配置
        let mut inbound_name_hash_list = HashSet::new();
        for inbound in &self.inbound {
            let name = inbound.name();
            if !inbound_name_hash_list.insert(name.to_string()) {
                return Err(anyhow::anyhow!("未知的入站名: {}", name));
            }
        }

        // 验证出站配置
        let mut outbound_name_hash_list = HashSet::new();
        for outbound in &self.outbound {
            let name = outbound.name();
            if !outbound_name_hash_list.insert(name.to_string()) {
                return Err(anyhow::anyhow!("未知的出站名: {}", name));
            }
        }

        // 验证路由引用
        if !outbound_name_hash_list.contains(&self.router.global) {
            return Err(anyhow::anyhow!(
                "全局出站 '{}' 没有找到",
                self.router.global
            ));
        }

        if !outbound_name_hash_list.contains(&self.router.default) {
            return Err(anyhow::anyhow!(
                "默认出站 '{}' 没有找到",
                self.router.default
            ));
        }

        // 验证路由规则
        for rule in &self.router.rule {
            if !outbound_name_hash_list.contains(&rule.outbound) {
                return Err(anyhow::anyhow!("规则出站 '{}' 没有找到", rule.outbound));
            }
        }

        // 验证路由规则集
        for rule_set in &self.router.rule_set {
            let outbound_name = match rule_set {
                router::RuleSet::RuleSetLocal(local) => &local.outbound,
                router::RuleSet::RuleSetRemote(remote) => &remote.outbound,
            };
            if !outbound_name_hash_list.contains(outbound_name) {
                warn!("规则集的出战 '{}' 没有找到", outbound_name);
            }
        }

        Ok(())
    }
}
