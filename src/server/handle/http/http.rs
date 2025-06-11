use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::trace;

use crate::server::handle::{copy_io, model::ServerTask};

pub async fn http(mut server_task: ServerTask) -> Result<()> {
    // 检查是否有请求数据
    let request_bytes = server_task
        .buffer
        .ok_or_else(|| anyhow::anyhow!("No request data available"))?;

    // 将字节数据转换为字符串（容忍无效UTF-8）
    let request_str = String::from_utf8_lossy(&request_bytes);

    trace!("{}", request_str);

    if request_str.starts_with("CONNECT") {
        // 发送 CONNECT 请求到代理服务器
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
    } else {
        server_task
            .outbound_stream
            .write_all(&request_bytes)
            .await?;
    }

    copy_io(server_task.inbound_stream, server_task.outbound_stream).await?;

    Ok(())
}
