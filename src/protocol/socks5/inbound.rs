use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, trace, warn};

use super::Socks5;

impl Socks5 {
    pub async fn listen(&self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;
        let http = Arc::new(self.clone());
        info!("SOCKS5 服务启动在: {}", &addr);
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let http = http.clone();
                    tokio::spawn(async move {
                        trace!("接收到连接, 来自: {} ", &addr);
                        if let Err(e) = http.accept(stream, addr).await {
                            warn!("入站 {} 处理 SOCKS5 连接失败: {}", http.name, e);
                        }
                    });
                }
                Err(e) => {
                    warn!("来自 {} 的连接失败: {}", addr, e);
                }
            }
        }
    }

    pub async fn accept(&self, stream: TcpStream, addr: SocketAddr) -> Result<()> {
        // 处理连接
        // 传递连接

        Ok(())
    }
}
