use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::error;

use crate::outbound::model::Outbound;

pub async fn copy_io<I, O>(inbound: I, outbound: O) -> Result<()>
where
    I: AsyncReadExt + AsyncWriteExt + Send + Unpin + 'static,
    O: AsyncReadExt + AsyncWriteExt + Send + Unpin + 'static,
{
    let (mut inbound_reader, mut inbound_writer) = tokio::io::split(inbound);
    let (mut outbound_reader, mut outbound_writer) = tokio::io::split(outbound);

    let inbound_to_outbound = tokio::io::copy(&mut inbound_reader, &mut outbound_writer);
    let outbound_to_inbound = tokio::io::copy(&mut outbound_reader, &mut inbound_writer);

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
        }
    }

    Ok(())
}

pub async fn handle_outbound(outbound: Outbound) -> Result<TcpStream> {
    let outbound_stream = match outbound {
        Outbound::Http(http) => http.outbound().await?,
        Outbound::Socks5(socks5) => socks5.outbound().await?,
    };
    Ok(outbound_stream)
}
