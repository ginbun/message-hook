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

    // `body` is the plain-text fallback per the Matrix spec — no markdown,
    // since not all clients render markdown in `body`.
    let plain_body = format!("{}\n\n{}", notification.title, notification.body);
    let html_title = html_escape(&notification.title);

    let formatted_body = if notification.fields.is_empty() {
        let html_body = html_escape(&notification.body).replace('\n', "<br>");
        format!("<h3>{}</h3><p>{}</p>", html_title, html_body)
    } else {
        // Render as `Key: value` lines rather than an HTML <table>: several
        // clients (e.g. Fractal) don't render tables and would collapse the
        // rows into a single line with no labels.
        let mut lines = String::new();
        for (i, (k, v)) in notification.fields.iter().enumerate() {
            if i > 0 {
                lines.push_str("<br>");
            }
            lines.push_str(&format!(
                "<b>{}</b>: {}",
                html_escape(k),
                html_escape(v).replace('\n', "<br>"),
            ));
        }
        format!("<h3>{}</h3><p>{}</p>", html_title, lines)
    };

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
