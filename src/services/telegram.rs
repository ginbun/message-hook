use crate::config::TelegramConfig;
use crate::models::Notification;
use reqwest::Client;
use serde_json::json;
use tracing::{error, info};

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub async fn send(client: &Client, config: &TelegramConfig, notification: &Notification) {
    if !config.enabled {
        return;
    }

    let url = format!("https://api.telegram.org/bot{}/sendMessage", config.token);

    let text = format!(
        "<b>{}</b>\n\n{}",
        html_escape(&notification.title),
        html_escape(&notification.body),
    );

    let body = json!({
        "chat_id": config.chat_id,
        "text": text,
        "parse_mode": "HTML"
    });

    match client.post(&url).json(&body).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                info!("Successfully sent message to Telegram");
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                error!("Failed to send message to Telegram: {} - {}", status, text);
            }
        }
        Err(e) => error!("Network error sending to Telegram: {}", e),
    }
}
