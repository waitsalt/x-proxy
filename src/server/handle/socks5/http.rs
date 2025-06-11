use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;

use crate::server::handle::{copy_io, model::ServerTask};

pub async fn http(mut server_task: ServerTask) -> Result<()> {
    let request_str = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
        server_task.target_host,
        server_task.target_port,
        server_task.target_host,
        server_task.target_port
    );
    server_task
        .outbound_stream
        .write_all(request_str.as_bytes())
        .await?;

    let mut request = Vec::new();
    let mut buf = [0u8; 1024];
    loop {
        let n = server_task.outbound_stream.read(&mut buf).await?;
        request.extend_from_slice(&buf[..n]);
        if n < 1024 {
            break;
        }
    }

    info!("{}", String::from_utf8(request).unwrap());

    // 发送成功响应给客户端
    let response = [0x05, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    server_task.inbound_stream.write_all(&response).await?;

    copy_io(server_task.inbound_stream, server_task.outbound_stream).await?;

    Ok(())
}
