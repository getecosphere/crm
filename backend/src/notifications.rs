use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
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

/// A short-lived service identity used when the CRM backend calls peer
/// domains that authenticate with the estate's shared JWT secret. The CRM
/// backend shares JWT_SECRET, so it can present a valid token without holding
/// a browser session.
#[derive(Debug, Serialize)]
struct ServiceClaims {
    sub: String,
    username: String,
    role: String,
    iat: i64,
    exp: i64,
}

fn service_token(config: &AppConfig) -> Result<String, String> {
    let now = Utc::now().timestamp();
    let claims = ServiceClaims {
        sub: "system:crm-backend".into(),
        username: "crm-backend".into(),
        role: "SYSTEM".into(),
        iat: now,
        exp: now + 120,
    };
    let header = Header::new(Algorithm::HS512);
    encode(&header, &claims, &EncodingKey::from_secret(config.jwt_secret.as_bytes()))
        .map_err(|e| format!("failed to sign service token: {e}"))
}

/// Pushes an in-app notification to the notifications domain. Fires and
/// forgets: a notification must never block the CRM write that triggered it.
pub async fn push(
    config: &AppConfig,
    notification: IngestNotification<'_>,
) -> Result<(), String> {
    let token = service_token(config)?;
    let client = reqwest::Client::new();
    let url = format!("{}/ingest", config.notifications_api_url.trim_end_matches('/'));
    let response = client
        .post(&url)
        .bearer_auth(&token)
        .json(&notification)
        .send()
        .await;
    let result = match response {
        Err(err) => Err(format!("notifications unreachable: {err}")),
        Ok(res) => {
            if !res.status().is_success() {
                let status = res.status();
                let text = res.text().await.unwrap_or_default();
                Err(format!("notifications ingest failed ({status}): {text}"))
            } else {
                match res.json::<IngestResponse>().await {
                    Ok(body) if body.ok => Ok(()),
                    Ok(_) => Err("notifications ingest returned ok=false".into()),
                    Err(err) => Err(format!("notifications bad response: {err}")),
                }
            }
        }
    };
    if let Err(err) = &result {
        tracing::warn!(kind = notification.kind, %err, "notification push failed");
    }
    result
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
