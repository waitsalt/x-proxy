use std::sync::Arc;

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info, trace};

use crate::{
    protocol::service::{self, handle_outbound},
    server::model::ServerConfig,
};

use super::model::Http;

impl Http {
    pub async fn inbound(&self, server_config: Arc<ServerConfig>) -> Result<()> {
        let listen = TcpListener::bind((self.host.clone(), self.port)).await?;
        info!("http 入站启动在: {}:{}", self.host, self.port);

        let http = Arc::new(self.clone());

        loop {
            match listen.accept().await {
                Ok((inbound_stream, addr)) => {
                    trace!("接受新连接: {}", addr);

                    let http = http.clone();
                    let server_config = server_config.clone();

                    tokio::spawn(async move {
                        if let Err(e) = http.handle_connect(inbound_stream, server_config).await {
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
        // 获取完整请求
        let mut request_buffer = Vec::new();
        let mut buffer_slice = [0u8; 1024];
        loop {
            let n = inbound_stream.read(&mut buffer_slice).await?;
            request_buffer.extend_from_slice(&buffer_slice[..n]);
            if n < 1024 {
                break;
            }
        }
        let request_str = String::from_utf8_lossy(&request_buffer);
        trace!("请求内容:\n{}", request_str);

        // 解析HTTP请求的目标地址和端口
        let (target_host, _target_port) = parse_http_request(&request_str)?;

        // 获取出站配置与连接
        let outbound_name = server_config.route_manager.switch(&target_host);
        let outbound = server_config.outbound_manager.get(outbound_name).unwrap();
        let mut outbound_stream = handle_outbound(outbound).await?;

        if request_str.starts_with("CONNECT") {
            inbound_stream
                .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                .await?;
        } else {
            outbound_stream.write_all(&request_buffer).await?;
        }

        service::copy_io(inbound_stream, outbound_stream).await?;

        Ok(())
    }
}

fn parse_http_request(request: &str) -> Result<(String, u16)> {
    // 从HTTP请求中解析出Host和端口
    let lines: Vec<&str> = request.lines().collect();
    for line in lines {
        if line.starts_with("Host:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let host_port = parts[1];
                let mut host = host_port.to_string();
                let mut port = 80; // 默认HTTP端口

                // 如果包含端口号，解析端口
                if let Some(colon_pos) = host_port.find(':') {
                    host = host_port[..colon_pos].to_string();
                    port = host_port[colon_pos + 1..].parse()?;
                }

                return Ok((host, port));
            }
        }
    }

    Err(anyhow::anyhow!("No Host header found in request"))
}
