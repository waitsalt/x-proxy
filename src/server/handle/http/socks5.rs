use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::error;

use crate::{
    protocol::model::HostType,
    server::handle::{copy_io, model::ServerTask},
};

pub async fn socks5(mut server_task: ServerTask) -> Result<()> {
    // 检查是否有请求数据
    let mut request_buffer = vec![];
    request_buffer.extend_from_slice(&[0x05, 0x01, 0x00]);
    match HostType::check(&server_task.target_host) {
        HostType::Ipv4 => {
            // IPv4
            request_buffer.push(0x01);
            for octet in server_task.target_host.split('.') {
                request_buffer.push(octet.parse::<u8>()?);
            }
        }
        HostType::Domain => {
            // Domain
            request_buffer.push(0x03);
            request_buffer.push(server_task.target_host.len() as u8);
            request_buffer.extend_from_slice(server_task.target_host.as_bytes());
        }
        HostType::Ipv6 => {
            // IPv6
            request_buffer.push(0x04);

            // 将 IPv6 地址字符串转换为 16 字节的数组
            let addr = server_task
                .target_host
                .split(':')
                .map(|part| u16::from_str_radix(part, 16).unwrap_or(0))
                .collect::<Vec<_>>();

            // 将每组 2 字节的十六进制数转换为大端字节序并添加到请求中
            for &value in &addr {
                request_buffer.extend_from_slice(&value.to_be_bytes());
            }
        }
    }
    request_buffer.extend_from_slice(&server_task.target_port.to_be_bytes());
    server_task
        .outbound_stream
        .write_all(&request_buffer)
        .await?;

    // 读取上游代理响应
    let mut response = [0u8; 4];
    server_task.outbound_stream.read_exact(&mut response).await?;

    if response[1] != 0x00 {
        error!("上游代理连接目标失败");
        let response = b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n";
        server_task.inbound_stream.write_all(response).await?;
        return Ok(());
    }

    // 跳过绑定地址和端口
    match response[3] {
        0x01 => {
            // IPv4
            let mut addr = [0u8; 4];
            server_task.outbound_stream.read_exact(&mut addr).await?;
        }
        0x03 => {
            // Domain
            let len = server_task.outbound_stream.read_u8().await?;
            let mut domain = vec![0u8; len as usize];
            server_task.outbound_stream.read_exact(&mut domain).await?;
        }
        0x04 => {
            // IPv6
            let mut addr = [0u8; 16];
            server_task.outbound_stream.read_exact(&mut addr).await?;
        }
        _ => return Err(anyhow::anyhow!("上游代理返回了不支持的地址类型")),
    }
    let mut port = [0u8; 2];
    server_task.outbound_stream.read_exact(&mut port).await?;

    // 检查是否有请求数据
    let request_bytes = server_task
        .buffer
        .ok_or_else(|| anyhow::anyhow!("No request data available"))?;

    // 将字节数据转换为字符串（容忍无效UTF-8）
    let request_str = String::from_utf8_lossy(&request_bytes);

    if request_str.starts_with("CONNECT") {
        server_task
            .inbound_stream
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await?;
    } else {
        server_task
            .outbound_stream
            .write_all(&request_bytes)
            .await?;
    }

    copy_io(server_task.inbound_stream, server_task.outbound_stream).await?;

    Ok(())
}
