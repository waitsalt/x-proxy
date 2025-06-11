pub mod http;
pub mod socks5;

pub mod model;

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::error;

pub async fn copy_io<I, O>(inbound: I, outbound: O) -> Result<()>
where
    I: AsyncReadExt + AsyncWriteExt + Send + Unpin + 'static,
    O: AsyncReadExt + AsyncWriteExt + Send + Unpin + 'static,
{
    let (mut inbound_reader, mut inbound_writer) = tokio::io::split(inbound);
    let (mut outbound_reader, mut outbound_writer) = tokio::io::split(outbound);

    let inbound_to_outbound = async {
        let result = tokio::io::copy(&mut inbound_reader, &mut outbound_writer).await;
        let _ = outbound_writer.shutdown().await;
        result
    };
    
    let outbound_to_inbound = async {
        let result = tokio::io::copy(&mut outbound_reader, &mut inbound_writer).await;
        let _ = inbound_writer.shutdown().await;
        result
    };

    tokio::select! {
        res = inbound_to_outbound => {
            if let Err(e) = res {
                error!("客户端到服务器传输错误: {}", e);
            }
        },
        res = outbound_to_inbound => {
            if let Err(e) = res {
                error!("服务器到客户端传输错误: {}", e);
            }
        },
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(300)) => {
            error!("连接超时，关闭连接");
        }
    }

    Ok(())
}
