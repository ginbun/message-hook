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

/// Matrix `org.matrix.custom.html` payload.
///
/// Many clients (Fractal, nheko, older Element) skip markdown in `body` and
/// also mishandle headings/tables. Stick to the widely-supported subset:
/// `strong`, `b`, `i`, `code`, `br`.
fn format_html(notification: &Notification) -> String {
    let title = format!("<strong>{}</strong>", html_escape(&notification.title));

    if notification.fields.is_empty() {
        let body = html_escape(&notification.body).replace('\n', "<br>");
        if body.is_empty() {
            return title;
        }
        return format!("{title}<br><br>{body}");
    }

    let mut html = title;
    html.push_str("<br><br>");
    for (i, (k, v)) in notification.fields.iter().enumerate() {
        if i > 0 {
            html.push_str("<br>");
        }
        html.push_str(&format_field_html(k, v));
    }
    html
}

fn format_field_html(key: &str, value: &str) -> String {
    let k = html_escape(key);
    let v = html_escape(value).replace('\n', "<br>");
    match key {
        "Note" => format!("<b>{k}</b>: <i>{v}</i>"),
        "Error" => format!("<b>{k}</b>: {v}"),
        _ => format!("<b>{k}</b>: <code>{v}</code>"),
    }
}

pub async fn send(client: &Client, config: &MatrixConfig, notification: &Notification) {
    if !config.enabled {
        return;
    }

    // Plain `body` is the fallback when a client ignores `formatted_body`.
    // Keep it text-only — no markdown markers.
    let plain_body = format!("{}\n\n{}", notification.title, notification.body);
    let formatted_body = format_html(notification);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn notification(title: &str, body: &str, fields: Vec<(&str, &str)>) -> Notification {
        Notification {
            title: title.into(),
            body: body.into(),
            fields: fields
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }

    #[test]
    fn html_uses_safe_tags_for_fields() {
        let n = notification(
            "[PROD] payments · 🔄 Sync Succeeded",
            "App: payments\nRevision: abc123",
            vec![
                ("App", "payments"),
                ("Revision", "abc123"),
                ("Health Status", "Healthy"),
                (
                    "Note",
                    "Sync succeeded means manifest apply succeeded. App health may still be Progressing.",
                ),
            ],
        );

        let html = format_html(&n);
        assert_eq!(
            html,
            "<strong>[PROD] payments · 🔄 Sync Succeeded</strong><br><br>\
             <b>App</b>: <code>payments</code><br>\
             <b>Revision</b>: <code>abc123</code><br>\
             <b>Health Status</b>: <code>Healthy</code><br>\
             <b>Note</b>: <i>Sync succeeded means manifest apply succeeded. App health may still be Progressing.</i>"
        );
        assert!(!html.contains("<h3>"));
        assert!(!html.contains("<table"));
        assert!(!html.contains('*'));
    }

    #[test]
    fn html_escapes_user_content() {
        let n = notification("app <b>", "unused", vec![("Error", "failed: a < b & c")]);

        let html = format_html(&n);
        assert_eq!(
            html,
            "<strong>app &lt;b&gt;</strong><br><br><b>Error</b>: failed: a &lt; b &amp; c"
        );
        assert!(!html.contains("app <b>"));
    }

    #[test]
    fn html_falls_back_to_plain_body() {
        let n = notification(
            "ArgoCD: my-app",
            "Status: Synced\nMessage: all good",
            vec![],
        );
        assert_eq!(
            format_html(&n),
            "<strong>ArgoCD: my-app</strong><br><br>Status: Synced<br>Message: all good"
        );
    }
}
