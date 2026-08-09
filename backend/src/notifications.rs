use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IngestNotification<'a> {
    pub recipient_ids: Vec<String>,
    pub kind: &'a str,
    pub title: &'a str,
    pub body: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IngestResponse {
    ok: bool,
    #[allow(dead_code)]
    notified: usize,
}

/// Pushes an in-app notification to the notifications domain. Fires and
/// forgets: a notification must never block the CRM write that triggered it.
pub async fn push(
    config: &AppConfig,
    notification: IngestNotification<'_>,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{}/ingest", config.notifications_api_url.trim_end_matches('/'));
    let response = client
        .post(&url)
        .json(&notification)
        .send()
        .await
        .map_err(|e| format!("notifications unreachable: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("notifications ingest failed ({status}): {text}"));
    }

    let body: IngestResponse = response
        .json()
        .await
        .map_err(|e| format!("notifications bad response: {e}"))?;
    if !body.ok {
        return Err("notifications ingest returned ok=false".into());
    }
    Ok(())
}

/// Human-readable name for a role, used in notification copy.
pub fn role_label(role: &str) -> &'static str {
    match role.to_uppercase().as_str() {
        "ADMIN" => "Administrator",
        "SALES_REP" => "Sales Representative",
        "PARTNER" => "Partner",
        _ => "User",
    }
}
