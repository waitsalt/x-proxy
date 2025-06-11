use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tracing::info;

use super::model::Socks5;

impl Socks5 {
    pub async fn outbound(&self) -> Result<TcpStream> {
        info!("出口为: {}", self.name);
        let mut outbound_stream = TcpStream::connect((self.host.as_str(), self.port)).await?;

        outbound_stream.write_all(&[0x05, 0x01, 0x00]).await?;

        // 检查是否需要认证
        self.handle_auth(&mut outbound_stream).await?;

        Ok(outbound_stream)
    }
}
