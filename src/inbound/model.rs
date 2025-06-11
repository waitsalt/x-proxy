use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::protocol::{http::model::Http, socks5::model::Socks5};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Inbound {
    Http(Http),
    Socks5(Socks5),
}

impl Inbound {
    pub fn name(&self) -> &str {
        match self {
            Inbound::Http(http) => &http.name,
            Inbound::Socks5(socks5) => &socks5.name,
        }
    }
}

pub struct InboundManager {
    pub inbound_hash_map: HashMap<String, Inbound>,
}

impl InboundManager {
    pub fn new() -> Self {
        Self {
            inbound_hash_map: HashMap::new(),
        }
    }

    pub fn init(inbound_vec: Vec<Inbound>) -> Self {
        let mut inbound_hash_map = HashMap::new();
        for inbound in inbound_vec {
            inbound_hash_map.insert(inbound.name().to_string(), inbound);
        }
        Self { inbound_hash_map }
    }

    pub fn get(&self, name: &str) -> Option<Inbound> {
        self.inbound_hash_map.get(name).cloned()
    }
}
