use anyhow::Result;
use tokio::net::TcpStream;
use tracing::trace;

use crate::{protocol::model::ProtocolType, server::handle};

pub struct ServerTask {
    pub buffer: Option<Vec<u8>>,
    pub target_host: String,
    pub target_port: u16,
    pub inbound_procotol: ProtocolType,
    pub outbound_procotol: ProtocolType,
    pub inbound_stream: TcpStream,
    pub outbound_stream: TcpStream,
}

impl ServerTask {
    pub fn new(
        buffer: Option<Vec<u8>>,
        target_host: &str,
        target_port: u16,
        inbound_procotol: ProtocolType,
        outbound_procotol: ProtocolType,
        inbound_stream: TcpStream,
        outbound_stream: TcpStream,
    ) -> Self {
        Self {
            buffer,
            target_host: target_host.to_string(),
            target_port,
            inbound_procotol,
            outbound_procotol,
            inbound_stream,
            outbound_stream,
        }
    }

    pub async fn handle(self) -> Result<()> {
        trace!(
            "{}:{:?} -> {:?}",
            self.target_host, self.inbound_procotol, self.outbound_procotol
        );
        match (&self.inbound_procotol, &self.outbound_procotol) {
            (ProtocolType::Http, ProtocolType::Http) => {
                handle::http::http(self).await?;
            }
            (ProtocolType::Http, ProtocolType::Socks5) => {
                handle::http::socks5(self).await?;
            }
            (ProtocolType::Socks5, ProtocolType::Http) => {
                handle::socks5::http(self).await?;
            }
            (ProtocolType::Socks5, ProtocolType::Socks5) => {
                handle::socks5::socks5(self).await?;
            }
        }
        Ok(())
    }
}
