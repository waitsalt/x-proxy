use serde::Deserialize;

use crate::common::config::route::{Rule, RuleSet, RuleType};

#[derive(Debug, Deserialize)]
pub struct Route {
    pub rule: Vec<Rule>,
    pub rule_set: Vec<RuleSet>,
    pub default: String,
}

impl Route {
    pub fn init(&self) -> Vec<Rule> {
        let mut rule_list = Vec::new();
        for rule_set in &self.rule_set {
            rule_list.extend(rule_set.init());
        }
        rule_list.extend(self.rule.clone());
        rule_list
    }
}

pub struct RouteManager {
    pub rule: Vec<Rule>,
    pub default: String,
}

impl RouteManager {
    pub fn init(rule: Vec<Rule>, default: String) -> Self {
        RouteManager { rule, default }
    }

    pub fn switch(&self, target_host: &str) -> &str {
        for rule in &self.rule {
            match rule.r#type {
                RuleType::Domain => {
                    if rule.source.contains(&target_host.to_string()) {
                        return &rule.outbound;
                    }
                }
                RuleType::Keyword => {
                    for keyword in &rule.source {
                        if target_host.contains(keyword) {
                            return &rule.outbound;
                        }
                    }
                }
            }
        }
        &self.default
    }
}
