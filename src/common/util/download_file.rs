use anyhow::Result;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use tracing::info;

fn download_file(url: &str, output_path: &str) -> Result<()> {
    // 1. 解析URL
    let (host, port, path) = parse_url(url)?;

    // 2. 建立TCP连接
    let mut stream = TcpStream::connect((host.as_str(), port))?;
    info!("Connected to {}:{}", host, port);

    // 3. 发送HTTP请求
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host
    );
    stream.write_all(request.as_bytes())?;

    // 4. 读取响应
    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;

    // 5. 分离头部和正文
    let (header, body) = separate_headers(&response)?;

    // 检查状态码
    if !header.starts_with("HTTP/1.1 200 OK") {
        return Err(anyhow::anyhow!(format!(
            "Server returned: {}",
            header.lines().next().unwrap_or("")
        )));
    }

    // 6. 保存文件
    File::create(output_path)?.write_all(body)?;
    info!("File saved to {}", output_path);

    Ok(())
}

// 辅助函数：解析URL
fn parse_url(url: &str) -> Result<(String, u16, String)> {
    let url = url.trim_start_matches("http://");
    let (host_port, path) = url.split_once('/').unwrap_or((url, ""));
    let path = if path.is_empty() { "/" } else { path };

    let (host, port) = match host_port.split_once(':') {
        Some((h, p)) => (h, p.parse()?),
        None => (host_port, 80),
    };

    Ok((host.to_string(), port, format!("/{}", path)))
}

// 辅助函数：分离HTTP头和正文
fn separate_headers(data: &[u8]) -> Result<(String, &[u8])> {
    let sep = b"\r\n\r\n";
    if let Some(pos) = data.windows(sep.len()).position(|w| w == sep) {
        let headers = std::str::from_utf8(&data[..pos])?.to_string();
        let body = &data[pos + sep.len()..];
        Ok((headers, body))
    } else {
        Err(anyhow::anyhow!(
            "Invalid HTTP response: no header separator"
        ))
    }
}
