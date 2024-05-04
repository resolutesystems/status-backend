use std::time::Duration;

use serde::Deserialize;
use serde_with::serde_as;
use tokio::fs;

const CONFIG_PATH: &str = "Config.toml";

pub async fn load_config() -> anyhow::Result<Config> {
    let contents = fs::read_to_string(CONFIG_PATH).await?;
    let config = toml::from_str(&contents)?;
    Ok(config)
}

#[derive(Clone, Deserialize)]
pub struct ApiConfig {
    pub bind: String,
}

#[derive(Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct CollectorConfig {
    pub redis_key: String,
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub interval: Duration,
    pub records: usize,
}

#[derive(Clone, Deserialize)]
pub struct ServiceConfig {
    pub label: String,
    pub endpoint: String,
}

#[derive(Clone, Deserialize)]
pub struct StatusConfig {
    pub services: Vec<ServiceConfig>,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub redis: RedisConfig,
    pub collector: CollectorConfig,
    pub status: StatusConfig,
}
