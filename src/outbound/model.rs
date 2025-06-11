use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::protocol::{http::model::Http, socks5::model::Socks5};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Outbound {
    Http(Http),
    Socks5(Socks5),
}

impl Outbound {
    pub fn name(&self) -> &str {
        match self {
            Outbound::Http(http) => &http.name,
            Outbound::Socks5(socks5) => &socks5.name,
        }
    }
}

pub struct OutboundManager {
    pub outbound_hash_map: HashMap<String, Outbound>,
    // pub inbound_stream_list: Vec<(ProtocolType, TcpStream)>,
}

impl OutboundManager {
    pub fn new() -> Self {
        Self {
            outbound_hash_map: HashMap::new(),
        }
    }

    pub fn init(outbound_vec: Vec<Outbound>) -> Self {
        let mut outbound_hash_map = HashMap::new();
        for outbound in outbound_vec {
            outbound_hash_map.insert(outbound.name().to_string(), outbound);
        }
        Self { outbound_hash_map }
    }

    pub fn get(&self, name: &str) -> Option<Outbound> {
        self.outbound_hash_map.get(name).cloned()
    }
}
