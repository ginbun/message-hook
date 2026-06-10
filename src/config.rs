use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ConfigError {}

impl ConfigError {
    fn not_found(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }

    fn message(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}
use reqwest::Client;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

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
        let path = if Path::new("config.toml").exists() {
            "config.toml"
        } else {
            "config"
        };

        let content = std::fs::read_to_string(path)
            .map_err(|_| ConfigError::not_found(format!("{path} not found")))?;

        let mut value: toml::Value = toml::from_str(&content)
            .map_err(|e| ConfigError::message(e.to_string()))?;
        merge_env_overrides(&mut value, "APP__");

        AppConfig::deserialize(value).map_err(|e| ConfigError::message(e.to_string()))
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

fn merge_env_overrides(root: &mut toml::Value, prefix: &str) {
    for (key, val) in std::env::vars() {
        let Some(rest) = key.strip_prefix(prefix) else {
            continue;
        };
        let path: Vec<String> = rest.split("__").map(|s| s.to_lowercase()).collect();
        set_toml_path(root, &path, parse_env_value(&val));
    }
}

fn set_toml_path(root: &mut toml::Value, path: &[String], val: toml::Value) {
    if path.is_empty() {
        return;
    }
    let table = root
        .as_table_mut()
        .expect("config root must be a TOML table");
    if path.len() == 1 {
        table.insert(path[0].clone(), val);
        return;
    }
    let entry = table
        .entry(path[0].clone())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));
    set_toml_path(entry, &path[1..], val);
}

fn parse_env_value(raw: &str) -> toml::Value {
    match raw {
        "true" => toml::Value::Boolean(true),
        "false" => toml::Value::Boolean(false),
        s => toml::Value::String(s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_example_toml() {
        let s = include_str!("../config.toml.example");
        let cfg: AppConfig = toml::from_str(s).expect("toml parse");
        assert!(cfg.targets.matrix.contains_key("default"));
        assert!(cfg.targets.telegram.contains_key("alerts"));
        assert_eq!(cfg.routes.len(), 2);
    }
}
