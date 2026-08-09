use axum::{http::StatusCode, Json};
use serde_json::json;

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn root() -> Json<serde_json::Value> {
    Json(json!({ "service": "crm-backend", "status": "ok" }))
}
