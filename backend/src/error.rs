use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Validation error")]
    Validation(Vec<(String, String)>),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal error: {0}")]
    Internal(#[source] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            AppError::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, json!({ "error": message }))
            }
            AppError::Validation(fields) => {
                let map: serde_json::Map<String, serde_json::Value> = fields
                    .iter()
                    .map(|(field, message)| {
                        (field.clone(), serde_json::Value::String(message.clone()))
                    })
                    .collect();
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    json!({ "error": "Validation failed", "fields": map }),
                )
            }
            AppError::Unauthorized(message) => {
                (StatusCode::UNAUTHORIZED, json!({ "error": message }))
            }
            AppError::Forbidden(message) => (StatusCode::FORBIDDEN, json!({ "error": message })),
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, json!({ "error": message })),
            AppError::Conflict(message) => (StatusCode::CONFLICT, json!({ "error": message })),
            AppError::Internal(error) => {
                tracing::error!(%error, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Internal server error" }),
                )
            }
        };
        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match &error {
            sqlx::Error::RowNotFound => AppError::NotFound("Record not found".into()),
            _ => AppError::Internal(anyhow::anyhow!(error)),
        }
    }
}

pub fn require_non_blank(fields: &[(&str, &str)]) -> AppResult<()> {
    let missing: Vec<(String, String)> = fields
        .iter()
        .filter(|(_, value)| value.trim().is_empty())
        .map(|(name, _)| (name.to_string(), "This field is required".to_string()))
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(missing))
    }
}
