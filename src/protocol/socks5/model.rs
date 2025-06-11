use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Socks5 {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub auth_enable: bool,
    pub username: Option<String>,
    pub password: Option<String>,
}
