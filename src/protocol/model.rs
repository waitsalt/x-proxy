use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use crate::outbound::model::Outbound;

use super::{http::model::Http, socks5::model::Socks5};

#[derive(Debug, Deserialize, Serialize, Clone)]
// #[serde(rename_all = "lowercase")]
pub enum ProtocolType {
    Http,
    Socks5,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Protocol {
    Direct,
    Http(Http),
    Socks5(Socks5),
}

pub enum HostType {
    Ipv4,
    Ipv6,
    Domain,
}

impl HostType {
    pub fn check(host: &str) -> Self {
        if let Ok(_ip) = host.parse::<std::net::Ipv4Addr>() {
            Self::Ipv4
        } else if let Ok(_ip) = host.parse::<std::net::Ipv6Addr>() {
            Self::Ipv6
        } else {
            Self::Domain
        }
    }
}
