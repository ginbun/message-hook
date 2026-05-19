use crate::config::MatrixConfig;
use crate::models::Notification;
use reqwest::Client;
use serde_json::json;
use tracing::{error, info};

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn encode_room_id(room_id: &str) -> String {
    room_id
        .chars()
        .flat_map(|c| match c {
            '!' => "%21".chars().collect::<Vec<_>>(),
            ':' => "%3A".chars().collect::<Vec<_>>(),
            '#' => "%23".chars().collect::<Vec<_>>(),
            _ => vec![c],
        })
        .collect()
}

pub async fn send(client: &Client, config: &MatrixConfig, notification: &Notification) {
    if !config.enabled {
        return;
    }

    let plain_body = format!("**{}**\n\n{}", notification.title, notification.body);
    let html_title = html_escape(&notification.title);
    let html_body = html_escape(&notification.body).replace('\n', "<br>");
    let formatted_body = format!("<h3>{}</h3><p>{}</p>", html_title, html_body);

    let body = json!({
        "msgtype": "m.text",
        "body": plain_body,
        "format": "org.matrix.custom.html",
        "formatted_body": formatted_body,
    });

    let base = config.homeserver.trim_end_matches('/');

    for room_id in &config.room_ids {
        let url = format!(
            "{}/_matrix/client/v3/rooms/{}/send/m.room.message",
            base,
            encode_room_id(room_id),
        );

        match client
            .post(&url)
            .bearer_auth(&config.token)
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("Successfully sent message to Matrix room {}", room_id);
                } else {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    error!(
                        "Failed to send message to Matrix room {}: {} - {}",
                        room_id, status, text
                    );
                }
            }
            Err(e) => error!("Network error sending to Matrix room {}: {}", room_id, e),
        }
    }
}
