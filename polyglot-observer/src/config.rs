use serde::Deserialize;
use config::{Config, ConfigError, File, Environment};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub base_log_path: String,
    pub lingo_api_url: String,
    pub lingo_api_key: String,
    pub loki_url: String,
    pub target_language: String,
    pub exclude_namespaces: Option<String>,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config"))
            .add_source(Environment::default())
            .build()?;

        s.try_deserialize()
    }
}
