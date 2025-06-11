use std::collections::HashMap;

use crate::common::config::inbound::Inbound;

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
