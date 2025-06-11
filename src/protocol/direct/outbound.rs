use anyhow::Result;
use tokio::net::TcpStream;

pub async fn new(target_addr: &str, target_port: u16) -> Result<TcpStream> {
    let stream = TcpStream::connect((target_addr, target_port)).await?;
    Ok(stream)
}
