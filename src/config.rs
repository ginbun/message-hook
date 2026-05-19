use config::{Config, ConfigError, Environment, File};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub targets: TargetsConfig,
    pub routes: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub addr: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub argocd_token: Option<String>,
    pub cloudflare_secret: Option<String>,
    pub alertmanager_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TargetsConfig {
    pub matrix: Option<MatrixConfig>,
    pub telegram: Option<TelegramConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MatrixConfig {
    pub enabled: bool,
    pub homeserver: String,
    pub token: String,
    pub room_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub token: String,
    pub chat_id: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}

/// Application state shared across all handlers.
pub struct AppState {
    pub config: AppConfig,
    pub http_client: Client,
}
