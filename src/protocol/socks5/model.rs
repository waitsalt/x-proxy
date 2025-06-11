use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Socks5 {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub auth_enable: bool,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Socks5 {
    pub async fn handle_auth(&self, stream: &mut TcpStream) -> Result<()> {
        let mut method_selection = [0u8; 2];
        stream.read(&mut method_selection).await?;

        if method_selection[0] != 0x05 {
            return Err(anyhow::anyhow!("不支持的SOCKS版本"));
        }

        let nmethods = method_selection[1] as usize;
        let mut methods = vec![0u8; nmethods];
        stream.read(&mut methods).await?;

        // 检查是否需要认证
        if self.auth_enable {
            // 查找客户端是否支持用户名/密码认证 (0x02)
            if methods.contains(&0x02) {
                // 回复使用用户名/密码认证方法
                stream.write_all(&[0x05, 0x02]).await?;
                stream.flush().await?;

                // 处理用户名/密码认证
                let mut auth_version = [0u8; 1];
                stream.read(&mut auth_version).await?;

                if auth_version[0] != 0x01 {
                    return Err(anyhow::anyhow!("不支持的认证版本"));
                }

                // 读取用户名
                let ulen = stream.read_u8().await? as usize;
                let mut username = vec![0u8; ulen];
                stream.read(&mut username).await?;
                let username = String::from_utf8(username)?;

                // 读取密码
                let plen = stream.read_u8().await? as usize;
                let mut password = vec![0u8; plen];
                stream.read(&mut password).await?;
                let password = String::from_utf8(password)?;

                // 验证用户名和密码
                if Some(username) == self.username && Some(password) == self.password {
                    // 认证成功
                    stream.write_all(&[0x01, 0x00]).await?;
                    stream.flush().await?;
                } else {
                    // 认证失败
                    stream.write_all(&[0x01, 0x01]).await?;
                    stream.flush().await?;
                    return Err(anyhow::anyhow!("认证失败"));
                }
            } else {
                // 客户端不支持我们需要的认证方法
                stream.write_all(&[0x05, 0xFF]).await?;
                stream.flush().await?;
                return Err(anyhow::anyhow!("客户端不支持所需的认证方法"));
            }
        } else {
            // 不需要认证，回复使用无认证方法
            stream.write_all(&[0x05, 0x00]).await?;
            stream.flush().await?;
        }
        Ok(())
    }
}
