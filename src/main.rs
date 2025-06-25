use anyhow::Result;
use tracing::error;
use x_proxy::{config::Config, service::ServiceConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置文件
    let config = Config::load()?;

    // 加载服务配置
    if let Err(e) = ServiceConfig::load(&config) {
        error!("{}", e);
    }

    // 启动服务
    if let Err(e) = ServiceConfig::init(&config).await {
        error!("{}", e);
    }

    // 等待 Ctrl+C 信号
    tokio::signal::ctrl_c().await?;

    Ok(())
}
