use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::CurrentUser,
    error::{AppError, AppResult},
    state::AppState,
};

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

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SalesRepStats {
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub user: UserRow,
    pub customers_created: i64,
    pub leads_submitted: i64,
    pub sales_resulted: i64,
    pub leads_assigned: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub role: Option<String>,
    pub search: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub auth_user_id: String,
    pub role: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub partner_company_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<String>,
    pub partner_company_id: Option<Uuid>,
}

pub async fn me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Json<UserRow>> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, auth_user_id, role, name, email, phone, status, partner_company_id, created_at \
         FROM users WHERE id = $1",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

pub async fn list_users(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<UserRow>>> {
    user.require_role(&["ADMIN"])?;
    let mut qb = sqlx::QueryBuilder::new(
        "SELECT id, auth_user_id, role, name, email, phone, status, partner_company_id, created_at \
         FROM users WHERE 1=1",
    );
    if let Some(role) = query.role.filter(|r| !r.trim().is_empty()) {
        qb.push(" AND role = ").push_bind(role.to_uppercase());
    }
    if let Some(status) = query.status.filter(|s| !s.trim().is_empty()) {
        qb.push(" AND status = ").push_bind(status.to_lowercase());
    }
    if let Some(search) = query.search.filter(|s| !s.trim().is_empty()) {
        let p1 = format!("%{search}%");
        let p2 = format!("%{search}%");
        qb.push(" AND (name ILIKE ")
            .push_bind(p1)
            .push(" OR email ILIKE ")
            .push_bind(p2)
            .push(")");
    }
    qb.push(" ORDER BY created_at DESC");
    let rows = qb.build_query_as::<UserRow>().fetch_all(&state.pool).await?;
    Ok(Json(rows))
}

pub async fn create_user(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserRow>)> {
    user.require_role(&["ADMIN"])?;
    let role = req.role.to_uppercase();
    if !["ADMIN", "SALES_REP", "PARTNER"].contains(&role.as_str()) {
        return Err(AppError::BadRequest("Invalid role".into()));
    }
    crate::error::require_non_blank(&[
        ("auth_user_id", &req.auth_user_id),
        ("name", &req.name),
        ("email", &req.email),
    ])?;
    if role == "PARTNER" && req.partner_company_id.is_none() {
        return Err(AppError::BadRequest(
            "partner_company_id is required for PARTNER users".into(),
        ));
    }
    if role != "PARTNER" && req.partner_company_id.is_some() {
        return Err(AppError::BadRequest(
            "partner_company_id is only allowed for PARTNER users".into(),
        ));
    }

    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE auth_user_id = $1")
            .bind(&req.auth_user_id)
            .fetch_optional(&state.pool)
            .await?;
    if existing.is_some() {
        return Err(AppError::Conflict(
            "This account is already provisioned in the CRM".into(),
        ));
    }

    let row = sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (auth_user_id, role, name, email, phone, status, partner_company_id) \
         VALUES ($1, $2, $3, $4, $5, 'active', $6) \
         RETURNING id, auth_user_id, role, name, email, phone, status, partner_company_id, created_at",
    )
    .bind(&req.auth_user_id)
    .bind(&role)
    .bind(req.name.trim())
    .bind(req.email.trim())
    .bind(req.phone.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.partner_company_id)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn get_user(
    State(state): State<AppState>,
    CurrentUser(current): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<SalesRepStats>> {
    let row = sqlx::query_as::<_, SalesRepStats>(
        "SELECT
            u.id, u.auth_user_id, u.role, u.name, u.email, u.phone, u.status, u.partner_company_id, u.created_at,
            (SELECT COUNT(*) FROM customers c WHERE c.created_by_user_id = u.id) AS customers_created,
            (SELECT COUNT(*) FROM leads l WHERE l.created_by_user_id = u.id) AS leads_submitted,
            (SELECT COUNT(*) FROM sales s WHERE s.registered_by_user_id = u.id) AS sales_resulted,
            (SELECT COUNT(*) FROM lead_assignments la JOIN leads l2 ON l2.id = la.lead_id WHERE l2.created_by_user_id = u.id) AS leads_assigned
         FROM users u WHERE u.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    if !current.is_admin() && current.id != row.user.id {
        return Err(AppError::Forbidden("Access denied".into()));
    }
    Ok(Json(row))
}

pub async fn update_user(
    State(state): State<AppState>,
    CurrentUser(current): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<UserRow>> {
    current.require_role(&["ADMIN"])?;

    if let Some(status) = &req.status {
        if !["active", "inactive"].contains(&status.as_str()) {
            return Err(AppError::BadRequest("Status must be active or inactive".into()));
        }
    }

    let row = sqlx::query_as::<_, UserRow>(
        "UPDATE users SET
            name = COALESCE($2, name),
            email = COALESCE($3, email),
            phone = COALESCE($4, phone),
            status = COALESCE($5, status),
            partner_company_id = $6,
            updated_at = now()
         WHERE id = $1
         RETURNING id, auth_user_id, role, name, email, phone, status, partner_company_id, created_at",
    )
    .bind(id)
    .bind(req.name.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.email.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.phone.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.status.as_deref().map(|s| s.to_lowercase()))
    .bind(req.partner_company_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(Json(row))
}
