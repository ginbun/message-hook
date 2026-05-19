use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct ArgoCDPayload {
    pub app_name: Option<String>,
    pub status: Option<String>,
    pub message: Option<String>,
    #[allow(dead_code)]
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CloudflarePayload {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub text: Option<String>,
    pub alert_type: Option<String>,
    pub alert_event: Option<String>,
    pub policy_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AlertmanagerPayload {
    pub status: String,
    pub alerts: Vec<Alert>,
    #[allow(dead_code)]
    pub common_annotations: serde_json::Value,
    #[allow(dead_code)]
    pub common_labels: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct Alert {
    pub status: String,
    pub labels: serde_json::Value,
    pub annotations: serde_json::Value,
    #[allow(dead_code)]
    pub starts_at: String,
    #[allow(dead_code)]
    pub ends_at: Option<String>,
}

impl From<ArgoCDPayload> for Notification {
    fn from(payload: ArgoCDPayload) -> Self {
        let body = format!(
            "App: {}\nStatus: {}\nMessage: {}",
            payload.app_name.unwrap_or_else(|| "Unknown App".to_string()),
            payload.status.unwrap_or_else(|| "N/A".to_string()),
            payload.message.unwrap_or_else(|| "No message provided".to_string())
        );
        Notification { body }
    }
}

impl From<CloudflarePayload> for Notification {
    fn from(payload: CloudflarePayload) -> Self {
        let body = format!(
            "Policy: {}\nType: {}\nEvent: {}\n\n{}",
            payload.policy_name.as_deref().unwrap_or("Unknown Policy"),
            payload.alert_type.as_deref().unwrap_or("N/A"),
            payload.alert_event.as_deref().unwrap_or("N/A"),
            payload.text.as_deref().unwrap_or(""),
        );
        Notification { body }
    }
}

impl From<AlertmanagerPayload> for Notification {
    fn from(payload: AlertmanagerPayload) -> Self {
        let mut body = format!(
            "Status: {} ({} alerts)\n",
            payload.status.to_uppercase(),
            payload.alerts.len()
        );

        for alert in payload.alerts.iter().take(5) {
            let summary = alert.annotations.get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("No summary");
            let instance = alert.labels.get("instance")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            
            body.push_str(&format!("- [{}] {}: {}\n", alert.status, instance, summary));
        }
        
        if payload.alerts.len() > 5 {
            body.push_str("... and more alerts");
        }

        Notification { body }
    }
}
