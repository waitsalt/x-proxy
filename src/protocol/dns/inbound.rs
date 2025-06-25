use anyhow::Result;
use tokio::net::UdpSocket;

use super::Dns;

impl Dns {
    pub async fn listen(&self) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let mut buf = [0; 1024];
        loop {
            let (len, addr) = socket.recv_from(&mut buf).await?;
            println!("{:?} bytes received from {:?}", len, addr);

            let len = socket.send_to(&buf[..len], addr).await?;
            println!("{:?} bytes sent", len);
        }
    }
}
