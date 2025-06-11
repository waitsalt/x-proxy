use serde::Deserialize;
use tracing::Level;

#[derive(Debug, Deserialize)]
pub struct Info {
    enable: bool,
    level: InfoLevel,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InfoLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl InfoLevel {
    pub fn to_tracing_level(&self) -> Level {
        match self {
            InfoLevel::Trace => Level::TRACE,
            InfoLevel::Debug => Level::DEBUG,
            InfoLevel::Info => Level::INFO,
            InfoLevel::Warn => Level::WARN,
            InfoLevel::Error => Level::ERROR,
        }
    }
}

impl Info {
    pub fn init(&self) {
        if self.enable {
            tracing_subscriber::fmt()
                .with_max_level(self.level.to_tracing_level())
                .init();
        }
    }
}
