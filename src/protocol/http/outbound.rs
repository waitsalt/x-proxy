use anyhow::Result;
use tokio::net::TcpStream;

use crate::protocol::http::model::Http;

impl Http {
    pub async fn outbound(&self) -> Result<TcpStream> {
        let outbound_stream = TcpStream::connect((self.host.as_str(), self.port)).await?;
        Ok(outbound_stream)
    }
}
