use std::collections::HashMap;

use crate::config::outbound::Outbound;

pub struct OutboundManager {
    pub outbound_hash_map: HashMap<String, Outbound>,
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

    // pub async fn handle(&self, outbound: Outbound) -> Result<(TcpStream, ProtocolType)> {
    //     let protocol_type;
    //     let outbound_stream = match outbound {
    //         Outbound::Http(http) => {
    //             protocol_type = ProtocolType::Http;
    //             http.outbound().await?
    //         }
    //         Outbound::Socks5(socks5) => {
    //             protocol_type = ProtocolType::Socks5;
    //             socks5.outbound().await?
    //         }
    //     };
    //     Ok((outbound_stream, protocol_type))
    // }
}
