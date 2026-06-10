pub mod matrix;
pub mod telegram;

use crate::config::TargetsConfig;
use crate::models::Notification;
use reqwest::Client;
use tracing::warn;

pub async fn dispatch(
    client: &Client,
    targets: &TargetsConfig,
    destinations: &[String],
    notification: Notification,
) {
    for dest in destinations {
        // Expected format: "<type>.<name>", e.g. "matrix.main" or "telegram.alerts"
        let Some((kind, name)) = dest.split_once('.') else {
            warn!("Invalid destination '{}': expected '<type>.<name>'", dest);
            continue;
        };

        match kind {
            "matrix" => {
                if let Some(config) = targets.matrix.get(name) {
                    let config = config.clone();
                    let notification = notification.clone();
                    let client = client.clone();
                    tokio::spawn(async move {
                        matrix::send(&client, &config, &notification).await;
                    });
                } else {
                    warn!("Matrix bot '{}' requested but not configured", name);
                }
            }
            "telegram" => {
                if let Some(config) = targets.telegram.get(name) {
                    let config = config.clone();
                    let notification = notification.clone();
                    let client = client.clone();
                    tokio::spawn(async move {
                        telegram::send(&client, &config, &notification).await;
                    });
                } else {
                    warn!("Telegram bot '{}' requested but not configured", name);
                }
            }
            _ => warn!("Unknown destination type '{}' in '{}'", kind, dest),
        }
    }
}
