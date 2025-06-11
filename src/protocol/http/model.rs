use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Http {
    pub name: String,
    pub host: String,
    pub port: u16,
}
