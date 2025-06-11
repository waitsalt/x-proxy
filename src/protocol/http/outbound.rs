use anyhow::Result;
use tokio::net::TcpStream;
use tracing::info;

use super::model::Http;

impl Http {
    pub async fn outbound(&self) -> Result<TcpStream> {
        info!("出口为: {}", self.name);
        let stream = TcpStream::connect((self.host.as_str(), self.port)).await?;
        Ok(stream)
    }
}
