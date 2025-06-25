pub mod dns;
pub mod http;
pub mod socks5;

pub enum Protocol {
    Http(http::Http),
    Socks5(socks5::Socks5),
}

pub enum ProtocolType {
    Http,
    Socks5,
}
