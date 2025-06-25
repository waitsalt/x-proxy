use serde::Deserialize;

use crate::protocol::{http::Http, socks5::Socks5};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
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
