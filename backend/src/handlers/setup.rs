use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::JwtUser,
    error::{AppError, AppResult},
    state::AppState,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupStatusResponse {
    pub initialized: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetupClaimRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserRow {
    pub id: Uuid,
    pub auth_user_id: String,
    pub role: String,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner_company_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Whether any administrator exists yet. `setup_claim` re-checks this under a
/// lock so two concurrent fresh installs cannot both claim the superadmin.
async fn initialized(pool: &sqlx::PgPool) -> Result<bool, sqlx::Error> {
    let exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE role = 'ADMIN')")
            .fetch_one(pool)
            .await?;
    Ok(exists.unwrap_or(false))
}

pub async fn setup_status(State(state): State<AppState>) -> AppResult<Json<SetupStatusResponse>> {
    let initialized = initialized(&state.pool).await?;
    Ok(Json(SetupStatusResponse { initialized }))
}

pub async fn setup_claim(
    State(state): State<AppState>,
    auth: JwtUser,
    Json(req): Json<SetupClaimRequest>,
) -> AppResult<(StatusCode, Json<UserRow>)> {
    crate::error::require_non_blank(&[("name", &req.name), ("email", &req.email)])?;

    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext('crm-initial-admin'))")
        .execute(&mut *tx)
        .await?;

    if initialized(&state.pool).await? {
        return Err(AppError::Conflict("CRM has already been initialized".into()));
    }

    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE auth_user_id = $1")
            .bind(&auth.user_id)
            .fetch_optional(&mut *tx)
            .await?;

    let user_id = match existing {
        Some(id) => {
            sqlx::query(
                "UPDATE users SET role = 'ADMIN', name = $1, email = $2, status = 'active', updated_at = now() WHERE id = $3",
            )
            .bind(req.name.trim())
            .bind(req.email.trim())
            .bind(id)
            .execute(&mut *tx)
            .await?;
            id
        }
        None => sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO users (auth_user_id, role, name, email, status) \
             VALUES ($1, 'ADMIN', $2, $3, 'active') RETURNING id",
        )
        .bind(&auth.user_id)
        .bind(req.name.trim())
        .bind(req.email.trim())
        .fetch_one(&mut *tx)
        .await?,
    };

    tx.commit().await?;

    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, auth_user_id, role, name, email, phone, status, partner_company_id, created_at \
         FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(user)))
}
