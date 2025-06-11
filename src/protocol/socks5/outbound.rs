use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::protocol::socks5::model::Socks5;

impl Socks5 {
    pub async fn outbound(&self) -> Result<TcpStream> {
        let mut outbound_stream = TcpStream::connect((self.host.as_str(), self.port)).await?;
        if self.auth_enable {
            outbound_stream.write_all(&[0x05, 0x01, 0x02]).await?;
            let mut response = [0u8; 2];
            outbound_stream.read_exact(&mut response).await?;
            if response[0] != 0x05 || response[1] != 0x02 {
                return Err(anyhow::anyhow!("代理不支持的格式或版本"));
            }
            match (&self.username, &self.password) {
                (Some(username), Some(password)) => {
                    let mut auth_buffer = vec![0x05];

                    let username_length = username.len() as u8;
                    auth_buffer.push(username_length);
                    auth_buffer.extend_from_slice(username.as_bytes());

                    let password_length = password.len() as u8;
                    auth_buffer.push(password_length);
                    auth_buffer.extend_from_slice(password.as_bytes());

                    outbound_stream.write_all(&auth_buffer).await?;

                    let mut response = [0u8; 2];
                    outbound_stream.read_exact(&mut response).await?;
                    if response[0] != 0x05 || response[1] != 0x00 {
                        return Err(anyhow::anyhow!("代理版本错误或认证失败"));
                    }
                }
                _ => {
                    return Err(anyhow::anyhow!("用户名或密码缺失"));
                }
            }
        } else {
            outbound_stream.write_all(&[0x05, 0x01, 0x00]).await?;
            let mut response = [0u8; 2];
            outbound_stream.read_exact(&mut response).await?;

            if response[0] != 0x05 || response[1] != 0x00 {
                return Err(anyhow::anyhow!("代理版本错误或认证失败"));
            }
        }

        Ok(outbound_stream)
    }
}
