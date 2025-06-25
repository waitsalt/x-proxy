use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};

use super::Http;

impl Http {
    pub async fn listen(&self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;
        let http = Arc::new(self.clone());
        info!("HTTP 服务启动在: {}", &addr);
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let http = http.clone();
                    tokio::spawn(async move {
                        debug!("接收到连接, 来自: {} ", &addr);
                        if let Err(e) = http.accept(stream, addr).await {
                            warn!("入站 {} 处理 HTTP 连接失败: {}", http.name, e);
                        }
                    });
                }
                Err(e) => {
                    warn!("来自 {} 的连接失败: {}", addr, e);
                }
            }
        }
    }

    pub async fn accept(&self, mut stream: TcpStream, addr: SocketAddr) -> Result<()> {
        // 处理连接
        // 传递连接

        let mut buffer = [0u8; 1024];
        let n = stream.peek(&mut buffer).await?;
        debug!(
            "接收到 {} 字节数据 {}",
            n,
            String::from_utf8_lossy(&buffer[..n])
        );

        Ok(())
    }
}
