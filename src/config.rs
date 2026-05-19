use config::{Config, ConfigError, Environment, File};
use reqwest::Client;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub sources: SourcesConfig,
    pub targets: TargetsConfig,
    #[serde(default)]
    pub routes: Vec<RouteRule>,
}

/// Routes webhook sources to notification targets (many-to-many).
#[derive(Debug, Deserialize, Clone)]
pub struct RouteRule {
    pub sources: Vec<String>,
    pub destinations: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub addr: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SourcesConfig {
    #[serde(default)]
    pub argocd: HashMap<String, ArgoCDSource>,
    #[serde(default)]
    pub cloudflare: HashMap<String, CloudflareSource>,
    #[serde(default)]
    pub alertmanager: HashMap<String, AlertmanagerSource>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ArgoCDSource {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CloudflareSource {
    pub secret: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AlertmanagerSource {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TargetsConfig {
    #[serde(default)]
    pub matrix: HashMap<String, MatrixConfig>,
    #[serde(default)]
    pub telegram: HashMap<String, TelegramConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MatrixConfig {
    pub enabled: bool,
    pub homeserver: String,
    pub token: String,
    pub room_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub token: String,
    pub chat_ids: Vec<String>,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        s.try_deserialize()
    }

    pub fn source_id(kind: &str, name: &str) -> String {
        format!("{kind}.{name}")
    }

    /// All destinations for a source id like `argocd.prod` (deduplicated, rule order preserved).
    pub fn destinations_for(&self, source: &str) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for rule in &self.routes {
            if rule.sources.iter().any(|s| s == source) {
                for dest in &rule.destinations {
                    if seen.insert(dest.clone()) {
                        result.push(dest.clone());
                    }
                }
            }
        }
        result
    }
}

/// Application state shared across all handlers.
pub struct AppState {
    pub config: AppConfig,
    pub http_client: Client,
}
