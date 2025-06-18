use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub tables: Vec<String>,
    pub meilisearch_indexes: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub meilisearch: meilisearch_sdk::client::Client,
    pub jwt_secret: String,
}

#[derive(Debug, Deserialize)]
struct TomlConfig {
    app: AppConfig,
}

impl AppConfig {
    pub fn new(env: &str) -> Self {
        let config_path = format!("config/{}.toml", env);
        let config_str = std::fs::read_to_string(&config_path)
            .unwrap_or_else(|_| panic!("Failed to read config file: {}", &config_path));
        let toml_config: TomlConfig = toml::from_str(&config_str)
            .expect("Failed to parse TOML config");
        toml_config.app
    }
}