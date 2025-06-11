use super::{http::model::Http, socks5::model::Socks5};

pub enum Protocol {
    Http(Http),
    Socks5(Socks5),
}

#[derive(Debug)]
pub enum ProtocolType {
    Http,
    Socks5,
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
