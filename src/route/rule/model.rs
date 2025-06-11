use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rule {
    pub r#type: RuleType,
    pub source: Vec<String>,
    pub target: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuleType {
    Domain,
    Keyword,
    // 其他可能的规则类型
}
