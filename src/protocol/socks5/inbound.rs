use std::sync::Arc;

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info};

use crate::{
    protocol::{
        model::HostType,
        service::{copy_io, handle_outbound},
    },
    server::model::ServerConfig,
};

use super::model::Socks5;

impl Socks5 {
    pub async fn inbound(&self, server_config: Arc<ServerConfig>) -> Result<()> {
        let tcp_listener = TcpListener::bind((self.host.as_str(), self.port)).await?;
        info!("socks5 入站启动在: {}:{}", self.host, self.port);

        let socks5 = Arc::new(self.clone());

        loop {
            match tcp_listener.accept().await {
                Ok((inbound_stream, addr)) => {
                    info!("接受新连接: {}", addr);

                    // 克隆配置
                    let socks5 = socks5.clone();
                    let server_config = server_config.clone();

                    // 处理连接
                    tokio::spawn(async move {
                        if let Err(e) = socks5.handle_connect(inbound_stream, server_config).await {
                            error!("解析连接错误: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("接受新连接错误: {}", e);
                }
            }
        }
    }

    pub async fn handle_connect(
        &self,
        mut inbound_stream: TcpStream,
        server_config: Arc<ServerConfig>,
    ) -> Result<()> {
        self.handle_auth(&mut inbound_stream).await?;

        // 读取SOCKS5请求
        let mut buf = [0u8; 4];
        inbound_stream.read(&mut buf).await?;

        if buf[0] != 0x05 || buf[1] != 0x01 {
            return Err(anyhow::anyhow!("不支持的SOCKS5命令"));
        }

        // 读取目标地址
        let addr_type = buf[3];
        let target_host = match addr_type {
            0x01 => {
                // IPv4
                let mut host = [0u8; 4];
                inbound_stream.read(&mut host).await?;
                format!("{}.{}.{}.{}", host[0], host[1], host[2], host[3])
            }
            0x03 => {
                // 域名
                let len = inbound_stream.read_u8().await? as usize;
                let mut domain = vec![0u8; len];
                inbound_stream.read(&mut domain).await?;
                String::from_utf8(domain)?
            }
            0x04 => {
                // IPv6
                let mut host = [0u8; 16];
                inbound_stream.read(&mut host).await?;
                format!(
                    "{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
                    host[0],
                    host[1],
                    host[2],
                    host[3],
                    host[4],
                    host[5],
                    host[6],
                    host[7],
                    host[8],
                    host[9],
                    host[10],
                    host[11],
                    host[12],
                    host[13],
                    host[14],
                    host[15]
                )
            }
            _ => return Err(anyhow::anyhow!("不支持的地址类型")),
        };

        // 读取端口
        let target_port = inbound_stream.read_u16().await?;
        // let _target_addr = format!("{}:{}", target_host, target_port);

        // 发送成功响应给客户端
        inbound_stream
            .write_all(&[0x05, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
            .await?;

        // 加载出站连接
        let outbound_name = server_config.route_manager.switch(&target_host);
        let outbound = server_config.outbound_manager.get(outbound_name).unwrap();
        let outbound_stream = match handle_outbound(outbound).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("代理连接失败: {}", e);
                // 发送失败响应
                let response = [0x05, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                inbound_stream.write_all(&response).await?;
                return Ok(());
            }
        };

        copy_io(inbound_stream, outbound_stream).await?;

        Ok(())
    }
}
