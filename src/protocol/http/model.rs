use anyhow::{Result, bail, ensure};
use bytes::Bytes;
use std::collections::HashMap;
use std::str;

#[derive(Debug, PartialEq, Clone)] // 添加 Debug, PartialEq, Clone 以便测试和使用
pub struct RequestHead {
    pub method: String,                   // HTTP方法 (GET, POST等)
    pub uri: String,                      // 请求URI/路径
    pub version: String,                  // HTTP版本 (HTTP/1.1等)
    pub headers: HashMap<String, String>, // 请求头集合
}

#[derive(Debug, PartialEq, Clone)] // 添加 Debug, PartialEq, Clone
pub struct ResponseHead {
    pub status: u16,                      // HTTP状态码 (200, 404等)
    pub version: String,                  // HTTP版本
    pub headers: HashMap<String, String>, // 响应头集合
}

pub struct BodyChunk {
    pub chunk: Bytes, // 二进制数据块
}

impl RequestHead {
    pub fn decode(bytes: &Bytes) -> Result<Self> {
        // 将字节流转换为UTF-8字符串进行处理
        let raw_str = str::from_utf8(bytes)?;

        let mut lines = raw_str.lines();

        // --- 1. 解析请求行 ---
        let request_line = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("无效请求：请求头为空"))?;

        let mut parts = request_line.split_whitespace();
        let method = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("无效请求行：缺少请求方法"))?
            .to_string();
        let uri = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("无效请求行：缺少 URI"))?
            .to_string();
        let version = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("无效请求行：缺少 HTTP 版本"))?
            .to_string();

        ensure!(parts.next().is_none(), "无效请求行：部分过多");

        // --- 2. 解析请求头字段 ---
        let mut headers = HashMap::new();
        for line in lines {
            // 到达结尾的空行，可以提前结束
            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                // 为了处理大小写不敏感，我们将键统一转换为小写
                let key_normalized = key.trim().to_lowercase();
                let value_trimmed = value.trim().to_string();

                // HashMap::insert 会自动处理覆盖。如果原始请求中有重复的头，只有最后一个会保留。
                headers.insert(key_normalized, value_trimmed);
            } else {
                bail!("无效的头字段格式: {}", line);
            }
        }

        Ok(RequestHead {
            method,
            uri,
            version,
            headers,
        })
    }

    pub fn encode(request_head: &RequestHead) -> Result<Bytes> {
        // 使用 Vec<u8> 作为缓冲区来构建字节序列
        let mut buffer = Vec::new();

        // --- 1. 编码请求行 ---
        // 格式化为 "METHOD URI VERSION\r\n"
        buffer.extend_from_slice(request_head.method.as_bytes());
        buffer.push(b' ');
        buffer.extend_from_slice(request_head.uri.as_bytes());
        buffer.push(b' ');
        buffer.extend_from_slice(request_head.version.as_bytes());
        buffer.extend_from_slice(b"\r\n");

        // --- 2. 编码请求头字段 ---
        // 注意：HashMap 的迭代顺序是不确定的
        for (key, value) in &request_head.headers {
            buffer.extend_from_slice(key.as_bytes());
            buffer.extend_from_slice(b": ");
            buffer.extend_from_slice(value.as_bytes());
            buffer.extend_from_slice(b"\r\n");
        }

        // --- 3. 添加结尾的空行 ---
        // 表示头部的结束
        buffer.extend_from_slice(b"\r\n");

        Ok(Bytes::from(buffer))
    }
}

impl ResponseHead {
    pub fn decode(bytes: &Bytes) -> Result<Self> {
        let raw_str = str::from_utf8(bytes)?;
        let mut lines = raw_str.lines();

        // --- 1. 解析状态行 ---
        let status_line = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("无效响应：响应头为空"))?;

        let mut parts = status_line.split_whitespace();
        let version = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("无效状态行：缺少 HTTP 版本"))?
            .to_string();
        let status_str = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("无效状态行：缺少状态码"))?;

        let status = status_str
            .parse::<u16>()
            .map_err(|_| anyhow::anyhow!("无效的状态码: {}", status_str))?;

        // Reason-Phrase (如 "OK", "Not Found") 是可选的，我们这里直接忽略。

        // --- 2. 解析头字段 ---
        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(':') {
                let key_normalized = key.trim().to_lowercase();
                let value_trimmed = value.trim().to_string();
                headers.insert(key_normalized, value_trimmed);
            } else {
                bail!("无效的头字段格式: {}", line);
            }
        }

        Ok(ResponseHead {
            version,
            status,
            headers,
        })
    }

    pub fn encode(response_head: &ResponseHead) -> Result<Bytes> {
        let mut buffer = Vec::new();

        // --- 1. 编码状态行 ---
        let reason_phrase = reason_phrase(response_head.status);
        // 格式化为 "VERSION STATUS REASON_PHRASE\r\n"
        buffer.extend_from_slice(response_head.version.as_bytes());
        buffer.push(b' ');
        buffer.extend_from_slice(response_head.status.to_string().as_bytes());
        buffer.push(b' ');
        buffer.extend_from_slice(reason_phrase.as_bytes());
        buffer.extend_from_slice(b"\r\n");

        // --- 2. 编码响应头字段 ---
        for (key, value) in &response_head.headers {
            buffer.extend_from_slice(key.as_bytes());
            buffer.extend_from_slice(b": ");
            buffer.extend_from_slice(value.as_bytes());
            buffer.extend_from_slice(b"\r\n");
        }

        // --- 3. 添加结尾的空行 ---
        buffer.extend_from_slice(b"\r\n");

        Ok(Bytes::from(buffer))
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown Status", // 对于未列出的状态码，提供一个通用短语
    }
}
