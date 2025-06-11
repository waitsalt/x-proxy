use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Http {
    pub name: String,
    pub host: String,
    pub port: u16,
}
