use anyhow::Result;

use x_proxy::{common, server};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化各模块
    common::init();

    let config = common::config::Config::init();

    server::init(&config).await?;

    // 等待推出
    tokio::signal::ctrl_c().await?;

    Ok(())
}
