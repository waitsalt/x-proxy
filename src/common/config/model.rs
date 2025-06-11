use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use tracing::warn;

use super::{inbound::Inbound, info::Info, mode::Mode, outbound::Outbound, route::Route};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub info: Info,
    pub mode: Mode,
    pub route: Route,
    pub inbound: Vec<Inbound>,
    pub outbound: Vec<Outbound>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let file_path = "config.yaml";
        let config_str = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read configuration file: {}", file_path))?;
        let config: Config = serde_yaml::from_str(&config_str)
            .with_context(|| "Failed to parse configuration file")?;
        
        config.validate()?;
        Ok(config)
    }

    pub fn init(&self) -> Result<()> {
        self.info.init();
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        // Validate inbound configurations
        let mut inbound_names = HashSet::new();
        for inbound in &self.inbound {
            let name = inbound.name();
            if !inbound_names.insert(name.to_string()) {
                return Err(anyhow::anyhow!("Duplicate inbound name: {}", name));
            }
        }

        // Validate outbound configurations
        let mut outbound_names = HashSet::new();
        for outbound in &self.outbound {
            let name = outbound.name();
            if !outbound_names.insert(name.to_string()) {
                return Err(anyhow::anyhow!("Duplicate outbound name: {}", name));
            }
        }

        // Validate route references
        if !outbound_names.contains(&self.route.global) {
            return Err(anyhow::anyhow!("Global outbound '{}' not found", self.route.global));
        }

        if !outbound_names.contains(&self.route.default) {
            return Err(anyhow::anyhow!("Default outbound '{}' not found", self.route.default));
        }

        // Validate rule outbound references
        for rule in &self.route.rule {
            if !outbound_names.contains(&rule.outbound) {
                return Err(anyhow::anyhow!("Rule outbound '{}' not found", rule.outbound));
            }
        }

        // Validate rule_set outbound references
        for rule_set in &self.route.rule_set {
            let outbound_name = match rule_set {
                super::route::RuleSet::RuleSetLocal(local) => &local.outbound,
                super::route::RuleSet::RuleSetRemote(remote) => &remote.outbound,
            };
            if !outbound_names.contains(outbound_name) {
                warn!("Rule set outbound '{}' not found", outbound_name);
            }
        }

        Ok(())
    }
}
