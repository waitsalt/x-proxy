use serde::Deserialize;

use crate::protocol::{http::Http, socks5::Socks5};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
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

    pub async fn init(&self) {
        let inbound = self.clone();
        // 监听服务为 loop 需要放在单独的工作携程中
        tokio::spawn(async move {
            match inbound {
                Inbound::Http(http) => http.listen().await,
                Inbound::Socks5(socks5) => socks5.listen().await,
            }
        });
    }
}
