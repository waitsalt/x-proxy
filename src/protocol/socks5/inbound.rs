use std::sync::Arc;

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info, trace};

use super::model::Socks5;
use crate::{
    protocol::model::ProtocolType,
    server::{handle::model::ServerTask, model::Server},
};

impl Socks5 {
    pub async fn listen(&self, server: Arc<Server>) -> Result<()> {
        let listener = TcpListener::bind((self.host.as_str(), self.port)).await?;
        info!("socks5 listener start in: {}:{}", self.host, self.port);

        let socks5 = Arc::new(self.clone());

        loop {
            match listener.accept().await {
                Ok((inbound_stream, source_addr)) => {
                    trace!("accept connect: {}", source_addr);
                    let server = server.clone();
                    let socks5 = socks5.clone();

                    tokio::spawn(async move {
                        if let Err(e) = socks5.inbound(inbound_stream, server).await {
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
        let mut buf = [0u8; 2];
        inbound_stream.read_exact(&mut buf).await?;
        if buf[0] != 0x05 {
            return Err(anyhow::anyhow!("不支持的socks版本"));
        }
        let method_number = buf[1] as usize;
        let mut method_list = vec![0u8; method_number];
        inbound_stream.read_exact(&mut method_list).await?;
        if self.auth_enable {
            if method_list.contains(&0x02) {
                inbound_stream.write_all(&[0x05, 0x02]).await?;
                inbound_stream.flush().await?;

                let mut auth_version = [0u8; 1];
                inbound_stream.read_exact(&mut auth_version).await?;
                if auth_version[0] != 0x01 {
                    return Err(anyhow::anyhow!("不支持的认证版本"));
                }
                
                // 用户名
                let mut username_len = [0u8; 1];
                inbound_stream.read_exact(&mut username_len).await?;
                if username_len[0] == 0 {
                    inbound_stream.write_all(&[0x01, 0x01]).await?;
                    return Err(anyhow::anyhow!("用户名长度为0"));
                }
                let mut username = vec![0u8; username_len[0] as usize];
                inbound_stream.read_exact(&mut username).await?;
                let username = String::from_utf8(username)
                    .map_err(|_| anyhow::anyhow!("用户名包含无效的UTF-8字符"))?;

                // 密码
                let mut password_len = [0u8; 1];
                inbound_stream.read_exact(&mut password_len).await?;
                if password_len[0] == 0 {
                    inbound_stream.write_all(&[0x01, 0x01]).await?;
                    return Err(anyhow::anyhow!("密码长度为0"));
                }
                let mut password = vec![0u8; password_len[0] as usize];
                inbound_stream.read_exact(&mut password).await?;
                let password = String::from_utf8(password)
                    .map_err(|_| anyhow::anyhow!("密码包含无效的UTF-8字符"))?;

                match (&self.username, &self.password) {
                    (Some(expected_username), Some(expected_password)) => {
                        if username == *expected_username && password == *expected_password {
                            inbound_stream.write_all(&[0x01, 0x00]).await?;
                        } else {
                            inbound_stream.write_all(&[0x01, 0x01]).await?;
                            return Err(anyhow::anyhow!("用户名或密码错误"));
                        }
                    }
                    _ => {
                        inbound_stream.write_all(&[0x01, 0x01]).await?;
                        return Err(anyhow::anyhow!("服务器未配置用户名或密码"));
                    }
                }
            } else {
                // 客户端不支持用户名/密码认证
                inbound_stream.write_all(&[0x05, 0xFF]).await?;
                inbound_stream.flush().await?;
                return Err(anyhow::anyhow!("客户端不支持所需的认证方法"));
            }
        } else {
            // 不需要认证，回复使用无认证方法
            inbound_stream.write_all(&[0x05, 0x00]).await?;
            inbound_stream.flush().await?;
        }

        // 读取SOCKS5请求
        let mut buf = [0u8; 4];
        inbound_stream.read_exact(&mut buf).await?;

        if buf[0] != 0x05 {
            return Err(anyhow::anyhow!("不支持的SOCKS版本"));
        }
        
        if buf[1] != 0x01 {
            // 发送错误响应：不支持的命令
            let error_response = [0x05, 0x07, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            inbound_stream.write_all(&error_response).await?;
            return Err(anyhow::anyhow!("不支持的SOCKS5命令: {}", buf[1]));
        }

        let addr_type = buf[3];
        let target_host = match addr_type {
            0x01 => {
                // IPv4
                let mut addr = [0u8; 4];
                inbound_stream.read_exact(&mut addr).await?;
                format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3])
            }
            0x03 => {
                // 域名
                let len = inbound_stream.read_u8().await? as usize;
                let mut domain = vec![0u8; len];
                inbound_stream.read_exact(&mut domain).await?;
                String::from_utf8(domain)?
            }
            0x04 => {
                // IPv6
                let mut addr = [0u8; 16];
                inbound_stream.read_exact(&mut addr).await?;
                format!(
                    "{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
                    addr[0],
                    addr[1],
                    addr[2],
                    addr[3],
                    addr[4],
                    addr[5],
                    addr[6],
                    addr[7],
                    addr[8],
                    addr[9],
                    addr[10],
                    addr[11],
                    addr[12],
                    addr[13],
                    addr[14],
                    addr[15]
                )
            }
            _ => return Err(anyhow::anyhow!("不支持的地址类型")),
        };

        // 读取端口
        let mut port_bytes = [0u8; 2];
        inbound_stream.read_exact(&mut port_bytes).await?;
        let target_port = u16::from_be_bytes(port_bytes);
        let outbound_name = server.route_manager.switch(&target_host);
        let outbound = server
            .outbound_manager
            .get(outbound_name)
            .ok_or_else(|| anyhow::anyhow!("找不到出站配置: {}", outbound_name))?;
        
        match server.outbound_manager.handle(outbound).await {
            Ok((outbound_stream, outbound_protocol)) => {
                let server_task = ServerTask::new(
                    None,
                    &target_host,
                    target_port,
                    ProtocolType::Socks5,
                    outbound_protocol,
                    inbound_stream,
                    outbound_stream,
                );

                server_task.handle().await?;
            }
            Err(e) => {
                // 发送连接失败响应
                let error_response = [0x05, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                let _ = inbound_stream.write_all(&error_response).await;
                return Err(anyhow::anyhow!("无法连接到出站代理: {}", e));
            }
        }

        Ok(())
    }
}
