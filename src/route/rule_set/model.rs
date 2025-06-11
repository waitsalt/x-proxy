use serde::{Deserialize, Serialize};

use crate::route::rule::model::Rule;

#[derive(Debug, Deserialize, Serialize)]
pub struct RuleSet {
    pub r#type: RuleSetType,
    pub name: String,
    pub path: String,
    pub target: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleSetType {
    Local,
    Remote,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuleSetFile {
    pub rule: Vec<Rule>,
}

impl RuleSet {
    pub fn init(&self) -> Vec<Rule> {
        match self.r#type {
            RuleSetType::Local => {
                let file_str = std::fs::read_to_string(&self.path).unwrap();
                let rule_set_file: RuleSetFile = serde_yaml::from_str(&file_str).unwrap();
                return rule_set_file.rule;
            }
            RuleSetType::Remote => {
                // Initialize remote rule set
                return vec![];
            }
        }
    }
}
