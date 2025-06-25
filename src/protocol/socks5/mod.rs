pub mod inbound;
pub mod outbound;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Socks5 {
    pub name: String,
    pub host: String,
    pub port: u16,
}
