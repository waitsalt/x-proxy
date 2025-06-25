use anyhow::Result;
use tokio::net::TcpStream;

use super::Socks5;

impl Socks5 {
    pub async fn connect(&self) -> Result<TcpStream> {
        let addr = format!("{}:{}", self.host, self.port);
        let stream = TcpStream::connect(addr).await?;
        Ok(stream)
    }
}
