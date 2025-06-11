use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Route {
    pub global: String,
    pub default: String,
    pub rule: Vec<Rule>,
    pub rule_set: Vec<RuleSet>,
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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RuleType {
    Domain,
    Keyword,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rule {
    pub r#type: RuleType,
    pub source: Vec<String>,
    pub outbound: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum RuleSet {
    RuleSetLocal(RuleSetLocal),
    RuleSetRemote(RuleSetRemote),
}

#[derive(Debug, Deserialize, Clone)]
pub struct RuleSetLocal {
    pub name: String,
    pub path: String,
    pub outbound: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct RuleSetRemote {
    pub name: String,
    pub path: String,
    pub url: String,
    pub outbound: String,
}

#[derive(Debug, Deserialize)]
pub struct RuleSetRule {
    pub r#type: RuleType,
    pub source: Vec<String>,
}

impl RuleSetRule {
    pub fn to_rule(self, outbound: &str) -> Rule {
        Rule {
            r#type: self.r#type,
            source: self.source,
            outbound: outbound.to_string(),
        }
    }
}

impl RuleSet {
    pub fn init(&self) -> Vec<Rule> {
        match self {
            RuleSet::RuleSetLocal(local) => {
                let mut rules = Vec::new();
                if let Ok(content) = std::fs::read_to_string(&local.path) {
                    let rule_set_rule_vec: Vec<RuleSetRule> =
                        serde_yaml::from_str(&content).unwrap();
                    for rule in rule_set_rule_vec {
                        rules.push(rule.to_rule(&local.outbound));
                    }
                }
                rules
            }
            RuleSet::RuleSetRemote(_remote) => {
                // let mut rules = Vec::new();
                // if let Ok(content) = reqwest::blocking::get(&remote.url).unwrap().text() {
                //     for line in content.lines() {
                //         if let Some(rule) = line.parse::<RuleSetRule>() {
                //             rules.push(rule.to_rule(&remote.outbound));
                //         }
                //     }
                // }
                // rules
                vec![]
            }
        }
    }
}
