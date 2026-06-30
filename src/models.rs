use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub title: String,
    pub body: String,
    /// Structured key/value pairs rendered as an HTML table by targets that
    /// support rich formatting (e.g. Matrix). Plain-text targets ignore this
    /// and fall back to `body`.
    #[serde(default)]
    pub fields: Vec<(String, String)>,
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
    /// Custom webhook: cloud region, e.g. india, us-east4.
    pub region: Option<String>,
    /// Custom webhook: human-readable area/continent label, e.g. "North America".
    #[serde(alias = "continent")]
    pub area: Option<String>,
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
        let app = payload.app_display().to_string();

        let title = if payload.is_custom_format() {
            let head = build_argocd_title_head(
                payload.environment.as_deref(),
                payload.region.as_deref(),
                payload.area.as_deref(),
            );
            // Always surface the app name so it's clear at a glance which app
            // the event is about.
            let prefix = if head.is_empty() {
                app.clone()
            } else {
                format!("{head} {app}")
            };
            match payload.event.as_deref() {
                Some(event) => format!("{prefix} — {event}"),
                None => prefix,
            }
        } else {
            format!("ArgoCD: {app}")
        };

        let (body, fields) = if payload.is_custom_format() {
            let mut fields: Vec<(String, String)> = Vec::new();
            if let Some(v) = payload.environment.as_deref() {
                fields.push(("Environment".into(), v.into()));
            }
            if let Some(v) = payload.region.as_deref() {
                fields.push(("Region".into(), v.into()));
            }
            if let Some(v) = payload.area.as_deref() {
                fields.push(("Area".into(), v.into()));
            }
            if let Some(v) = payload.event.as_deref() {
                fields.push(("Event".into(), v.into()));
            }
            if !app.is_empty() {
                fields.push(("App".into(), app.clone()));
            }
            if let Some(v) = payload.revision.as_deref() {
                fields.push(("Revision".into(), v.into()));
            }
            if let Some(v) = payload.health_status.as_deref() {
                fields.push(("Health".into(), v.into()));
            }
            if let Some(v) = payload.operation_phase.as_deref() {
                fields.push(("Operation".into(), v.into()));
            }
            if let Some(v) = payload.project.as_deref() {
                fields.push(("Project".into(), v.into()));
            }
            if let Some(v) = payload.namespace.as_deref() {
                fields.push(("Namespace".into(), v.into()));
            }
            if let Some(v) = payload.sync_mode.as_deref() {
                fields.push(("Sync Mode".into(), v.into()));
            }
            if let Some(v) = payload.message.as_deref() {
                fields.push(("Message".into(), v.into()));
            }

            let body = if fields.is_empty() {
                "No details provided".to_string()
            } else {
                fields
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            (body, fields)
        } else {
            let body = format!(
                "Status: {}\nMessage: {}",
                payload.status.as_deref().unwrap_or("N/A"),
                payload.message.as_deref().unwrap_or("No message provided")
            );
            (body, Vec::new())
        };

        Notification { title, body, fields }
    }
}

/// Build the leading `ENV (region[, area])` portion of an ArgoCD title.
fn build_argocd_title_head(env: Option<&str>, region: Option<&str>, area: Option<&str>) -> String {
    let mut head = String::new();
    if let Some(e) = env {
        head.push_str(e);
    }
    if let Some(r) = region {
        let loc = match area {
            Some(a) => format!("{r}, {a}"),
            None => r.to_string(),
        };
        if head.is_empty() {
            head.push_str(&format!("({loc})"));
        } else {
            head.push_str(&format!(" ({loc})"));
        }
    }
    head
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argocd_custom_sync_succeeded() {
        let payload: ArgoCDPayload = serde_json::from_str(
            r#"{
              "environment": "TEST",
              "region": "india",
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
        assert_eq!(n.title, "TEST (india) composite-admin-test — sync-succeeded");
        assert!(n.body.contains("Event: sync-succeeded"));
        assert!(n.body.contains("Health: Healthy"));
        assert!(n.body.contains("Revision: a1b2c3d4e5f6789012345678901234567890abcd"));
        assert!(n.body.contains("Namespace: composite-test"));
        assert!(n.fields.iter().any(|(k, v)| k == "Region" && v == "india"));
    }

    #[test]
    fn argocd_custom_sync_failed() {
        let payload: ArgoCDPayload = serde_json::from_str(
            r#"{
              "environment": "PROD",
              "region": "us-east4",
              "area": "North America",
              "event": "sync-failed",
              "app": "composite-admin-prod",
              "healthStatus": "Degraded",
              "operationPhase": "Failed"
            }"#,
        )
        .unwrap();

        let n: Notification = payload.into();
        assert_eq!(
            n.title,
            "PROD (us-east4, North America) composite-admin-prod — sync-failed"
        );
        assert!(n.body.contains("Operation: Failed"));
        assert!(n.fields.iter().any(|(k, v)| k == "Area" && v == "North America"));
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
        Notification { title, body, fields: Vec::new() }
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
        
        Notification { title, body, fields: Vec::new() }
    }
}
