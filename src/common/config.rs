use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{inbound::model::Inbound, outbound::model::Outbound, route::model::Route};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub info: Info,
    pub route: Route,
    pub model: Model,
    pub inbound: Vec<Inbound>,
    pub outbound: Vec<Outbound>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    pub enable: bool,
    pub level: InfoLevel,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InfoLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Model {
    Direct,
    Global,
    Rule,
}

impl Config {
    pub fn init() -> Self {
        let config_path = Path::new("config.yaml");
        let content = std::fs::read_to_string(config_path).unwrap();
        let config: Config = serde_yaml::from_str(&content).unwrap();

        config
    }
}
