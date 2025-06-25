pub mod inbound;
pub mod model;

pub struct Dns {
    pub name: String,
    pub host: String,
    pub port: u16,
}

impl Dns {
    pub fn new(name: String, host: String, port: u16) -> Self {
        Dns { name, host, port }
    }
}
