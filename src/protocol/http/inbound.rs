use std::sync::Arc;

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info, trace};

use super::model::Http;
use crate::{
    protocol::model::ProtocolType,
    server::{handle::model::ServerTask, model::Server},
};

impl Http {
    pub async fn listen(&self, server: Arc<Server>) -> Result<()> {
        let listener = TcpListener::bind((self.host.as_str(), self.port)).await?;
        info!("http listener start in: {}:{}", self.host, self.port);

        let http = Arc::new(self.clone());

        loop {
            match listener.accept().await {
                Ok((inbound_stream, source_addr)) => {
                    trace!("accept connect: {}", source_addr);
                    let server = server.clone();
                    let http = http.clone();

                    tokio::spawn(async move {
                        if let Err(e) = http.inbound(inbound_stream, server).await {
                            error!("inbound handle error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("listenter accept error: {}", e)
                }
            }
        }
    }

    pub async fn inbound(&self, mut inbound_stream: TcpStream, server: Arc<Server>) -> Result<()> {
        // 获取完整请求
        let mut request = Vec::new();
        let mut buf = [0u8; 1024];
        loop {
            let n = inbound_stream.read(&mut buf).await?;
            request.extend_from_slice(&buf[..n]);
            if n < 1024 {
                break;
            }
        }

        let request_str = String::from_utf8_lossy(&request);
        trace!("HTTP request: {}", request_str);

        let (target_host, target_port) = match parse_http_request(&request_str) {
            Ok(parsed) => parsed,
            Err(e) => {
                // Send 400 Bad Request response
                let error_response = b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
                let _ = inbound_stream.write_all(error_response).await;
                return Err(e);
            }
        };

        let outbound_name = server.route_manager.switch(&target_host);
        let outbound = match server.outbound_manager.get(outbound_name) {
            Some(ob) => ob,
            None => {
                let error_response = b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n";
                let _ = inbound_stream.write_all(error_response).await;
                return Err(anyhow::anyhow!("找不到出站配置: {}", outbound_name));
            }
        };

        match server.outbound_manager.handle(outbound).await {
            Ok((outbound_stream, outbound_protocol)) => {
                let server_task = ServerTask::new(
                    Some(request),
                    &target_host,
                    target_port,
                    ProtocolType::Http,
                    outbound_protocol,
                    inbound_stream,
                    outbound_stream,
                );

                server_task.handle().await?;
            }
            Err(e) => {
                // Send 502 Bad Gateway response
                let error_response = b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n";
                let _ = inbound_stream.write_all(error_response).await;
                return Err(anyhow::anyhow!("无法连接到出站代理: {}", e));
            }
        }

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
