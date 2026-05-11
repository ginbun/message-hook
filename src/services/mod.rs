pub mod matrix;
pub mod telegram;

use crate::config::TargetsConfig;
use crate::models::Notification;
use reqwest::Client;
use tracing::warn;

pub async fn dispatch(
    client: &Client,
    targets: &TargetsConfig,
    route: &[String],
    notification: Notification,
) {
    for target_name in route {
        match target_name.as_str() {
            "matrix" => {
                if let Some(ref config) = targets.matrix {
                    let config = config.clone();
                    let notification = notification.clone();
                    let client = client.clone();
                    tokio::spawn(async move {
                        matrix::send(&client, &config, &notification).await;
                    });
                } else {
                    warn!("Matrix target requested but not configured");
                }
            }
            "telegram" => {
                if let Some(ref config) = targets.telegram {
                    let config = config.clone();
                    let notification = notification.clone();
                    let client = client.clone();
                    tokio::spawn(async move {
                        telegram::send(&client, &config, &notification).await;
                    });
                } else {
                    warn!("Telegram target requested but not configured");
                }
            }
            _ => warn!("Unknown target: {}", target_name),
        }
    }
}
