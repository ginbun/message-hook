use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct ArgoCDPayload {
    /// Legacy Argo CD notifications field; also accepts `"app"`.
    #[serde(alias = "app")]
    pub app_name: Option<String>,
    /// Legacy status field.
    pub status: Option<String>,
    pub message: Option<String>,
    /// Custom webhook: TEST / PROD from overlay context.
    pub environment: Option<String>,
    /// Custom webhook: e.g. sync-succeeded, sync-failed.
    pub event: Option<String>,
    pub revision: Option<String>,
    #[serde(rename = "healthStatus")]
    pub health_status: Option<String>,
    #[serde(rename = "operationPhase")]
    pub operation_phase: Option<String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    #[serde(rename = "syncMode")]
    pub sync_mode: Option<String>,
}

impl ArgoCDPayload {
    fn is_custom_format(&self) -> bool {
        self.event.is_some()
            || self.environment.is_some()
            || self.revision.is_some()
            || self.health_status.is_some()
    }

    fn app_display(&self) -> &str {
        self.app_name.as_deref().unwrap_or("Unknown App")
    }
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
        let app = payload.app_display();

        let title = match (&payload.environment, &payload.event) {
            (Some(env), Some(event)) => format!("ArgoCD [{env}]: {app} — {event}"),
            (Some(env), None) => format!("ArgoCD [{env}]: {app}"),
            (None, Some(event)) => format!("ArgoCD: {app} — {event}"),
            (None, None) => format!("ArgoCD: {app}"),
        };

        let body = if payload.is_custom_format() {
            let mut lines = Vec::new();
            if let Some(ref v) = payload.event {
                lines.push(format!("Event: {v}"));
            }
            if let Some(ref v) = payload.health_status {
                lines.push(format!("Health: {v}"));
            }
            if let Some(ref v) = payload.operation_phase {
                lines.push(format!("Operation: {v}"));
            }
            if let Some(ref v) = payload.revision {
                let short = if v.len() > 7 { &v[..7] } else { v.as_str() };
                lines.push(format!("Revision: {short}"));
            }
            if let Some(ref v) = payload.project {
                lines.push(format!("Project: {v}"));
            }
            if let Some(ref v) = payload.namespace {
                lines.push(format!("Namespace: {v}"));
            }
            if let Some(ref v) = payload.sync_mode {
                lines.push(format!("Sync Mode: {v}"));
            }
            if let Some(ref v) = payload.message {
                lines.push(format!("Message: {v}"));
            }
            if lines.is_empty() {
                "No details provided".to_string()
            } else {
                lines.join("\n")
            }
        } else {
            format!(
                "Status: {}\nMessage: {}",
                payload.status.as_deref().unwrap_or("N/A"),
                payload.message.as_deref().unwrap_or("No message provided")
            )
        };

        Notification { title, body }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argocd_custom_sync_succeeded() {
        let payload: ArgoCDPayload = serde_json::from_str(
            r#"{
              "environment": "TEST",
              "event": "sync-succeeded",
              "app": "composite-admin-test",
              "revision": "a1b2c3d4e5f6789012345678901234567890abcd",
              "healthStatus": "Healthy",
              "operationPhase": "Succeeded",
              "project": "test-project",
              "namespace": "composite-test",
              "syncMode": "AUTO"
            }"#,
        )
        .unwrap();

        let n: Notification = payload.into();
        assert_eq!(n.title, "ArgoCD [TEST]: composite-admin-test — sync-succeeded");
        assert!(n.body.contains("Event: sync-succeeded"));
        assert!(n.body.contains("Health: Healthy"));
        assert!(n.body.contains("Revision: a1b2c3d"));
        assert!(n.body.contains("Namespace: composite-test"));
    }

    #[test]
    fn argocd_custom_sync_failed() {
        let payload: ArgoCDPayload = serde_json::from_str(
            r#"{
              "environment": "PROD",
              "event": "sync-failed",
              "app": "composite-admin-prod",
              "healthStatus": "Degraded",
              "operationPhase": "Failed"
            }"#,
        )
        .unwrap();

        let n: Notification = payload.into();
        assert_eq!(n.title, "ArgoCD [PROD]: composite-admin-prod — sync-failed");
        assert!(n.body.contains("Operation: Failed"));
    }

    #[test]
    fn argocd_legacy_format() {
        let payload: ArgoCDPayload = serde_json::from_str(
            r#"{"app_name":"my-app","status":"Synced","message":"all good"}"#,
        )
        .unwrap();

        let n: Notification = payload.into();
        assert_eq!(n.title, "ArgoCD: my-app");
        assert!(n.body.contains("Status: Synced"));
        assert!(n.body.contains("Message: all good"));
    }
}

impl From<CloudflarePayload> for Notification {
    fn from(payload: CloudflarePayload) -> Self {
        let title = format!(
            "Cloudflare Alert: {}",
            payload.policy_name.as_deref().unwrap_or("Unknown Policy")
        );
        let body = format!(
            "Type: {}\nEvent: {}\n\n{}",
            payload.alert_type.as_deref().unwrap_or("N/A"),
            payload.alert_event.as_deref().unwrap_or("N/A"),
            payload.text.as_deref().unwrap_or(""),
        );
        Notification { title, body }
    }
}

impl From<AlertmanagerPayload> for Notification {
    fn from(payload: AlertmanagerPayload) -> Self {
        let title = format!("Alertmanager: {} ({})", payload.status.to_uppercase(), payload.alerts.len());
        let mut body = String::new();
        
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
        
        Notification { title, body }
    }
}
