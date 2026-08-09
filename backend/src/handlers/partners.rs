use axum::{
    extract::{Path, State},
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
pub struct PartnerRow {
    pub id: Uuid,
    pub name: String,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PartnerListItem {
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub partner: PartnerRow,
    pub categories: Vec<String>,
    pub assigned_leads: i64,
    pub sales_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRow {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PartnerPerformance {
    pub assigned: i64,
    pub new_count: i64,
    pub contacted: i64,
    pub in_progress: i64,
    pub sales_count: i64,
    pub no_sale: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePartnerRequest {
    pub name: String,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePartnerRequest {
    pub name: Option<String>,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryIdsRequest {
    pub product_category_ids: Vec<Uuid>,
}

pub async fn list_partners(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Json<Vec<PartnerListItem>>> {
    user.require_role(&["ADMIN"])?;
    let rows = sqlx::query_as::<_, PartnerListItem>(
        "SELECT
            p.id, p.name, p.contact_person, p.email, p.phone, p.address, p.status, p.created_at,
            COALESCE(ARRAY(
                SELECT pc.name FROM partner_product_categories ppc
                JOIN product_categories pc ON pc.id = ppc.product_category_id
                WHERE ppc.partner_company_id = p.id AND pc.status = 'active'
            ), '{}') AS categories,
            (SELECT COUNT(*) FROM lead_assignments la WHERE la.partner_company_id = p.id) AS assigned_leads,
            (SELECT COUNT(*) FROM sales s JOIN lead_assignments la2 ON la2.id = s.lead_assignment_id WHERE la2.partner_company_id = p.id) AS sales_count
         FROM partner_companies p ORDER BY p.created_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

pub async fn create_partner(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<CreatePartnerRequest>,
) -> AppResult<(StatusCode, Json<PartnerRow>)> {
    user.require_role(&["ADMIN"])?;
    crate::error::require_non_blank(&[("name", &req.name)])?;
    let row = sqlx::query_as::<_, PartnerRow>(
        "INSERT INTO partner_companies (name, contact_person, email, phone, address, status) \
         VALUES ($1, $2, $3, $4, $5, 'active') \
         RETURNING id, name, contact_person, email, phone, address, status, created_at",
    )
    .bind(req.name.trim())
    .bind(req.contact_person.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.email.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.phone.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.address.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn update_partner(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePartnerRequest>,
) -> AppResult<Json<PartnerRow>> {
    user.require_role(&["ADMIN"])?;
    if let Some(status) = &req.status {
        if !["active", "inactive"].contains(&status.as_str()) {
            return Err(AppError::BadRequest("Status must be active or inactive".into()));
        }
    }
    let row = sqlx::query_as::<_, PartnerRow>(
        "UPDATE partner_companies SET
            name = COALESCE($2, name),
            contact_person = COALESCE($3, contact_person),
            email = COALESCE($4, email),
            phone = COALESCE($5, phone),
            address = COALESCE($6, address),
            status = COALESCE($7, status),
            updated_at = now()
         WHERE id = $1
         RETURNING id, name, contact_person, email, phone, address, status, created_at",
    )
    .bind(id)
    .bind(req.name.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.contact_person.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.email.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.phone.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.address.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.status.as_deref().map(|s| s.to_lowercase()))
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Partner not found".into()))?;
    Ok(Json(row))
}

pub async fn get_partner(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_role(&["ADMIN"])?;
    let partner = sqlx::query_as::<_, PartnerRow>(
        "SELECT id, name, contact_person, email, phone, address, status, created_at \
         FROM partner_companies WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Partner not found".into()))?;

    let categories = sqlx::query_as::<_, CategoryRow>(
        "SELECT pc.id, pc.name FROM partner_product_categories ppc
         JOIN product_categories pc ON pc.id = ppc.product_category_id
         WHERE ppc.partner_company_id = $1 AND pc.status = 'active' ORDER BY pc.name",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let performance = sqlx::query_as::<_, PartnerPerformance>(
        "SELECT
            COUNT(*)::bigint AS assigned,
            COUNT(*) FILTER (WHERE status = 'NEW')::bigint AS new_count,
            COUNT(*) FILTER (WHERE status = 'CONTACTED')::bigint AS contacted,
            COUNT(*) FILTER (WHERE status = 'IN_PROGRESS')::bigint AS in_progress,
            COUNT(*) FILTER (WHERE status = 'SALE')::bigint AS sales_count,
            COUNT(*) FILTER (WHERE status = 'NO_SALE')::bigint AS no_sale
         FROM lead_assignments WHERE partner_company_id = $1",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    let partner_users = sqlx::query_as::<_, UserSummary>(
        "SELECT id, auth_user_id, role, name, email, phone, status, partner_company_id, created_at \
         FROM users WHERE partner_company_id = $1",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "partner": partner,
        "categories": categories,
        "performance": performance,
        "users": partner_users,
    })))
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserSummary {
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

pub async fn set_partner_categories(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CategoryIdsRequest>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_role(&["ADMIN"])?;
    let exists: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM partner_companies WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;
    if exists.is_none() {
        return Err(AppError::NotFound("Partner not found".into()));
    }

    let mut tx = state.pool.begin().await?;
    sqlx::query("DELETE FROM partner_product_categories WHERE partner_company_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for category_id in &req.product_category_ids {
        let exists: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM product_categories WHERE id = $1 AND status = 'active'")
                .bind(category_id)
                .fetch_optional(&mut *tx)
                .await?;
        if let Some(_) = exists {
            sqlx::query(
                "INSERT INTO partner_product_categories (partner_company_id, product_category_id) \
                 VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(id)
            .bind(category_id)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;

    let categories = sqlx::query_as::<_, CategoryRow>(
        "SELECT pc.id, pc.name FROM partner_product_categories ppc
         JOIN product_categories pc ON pc.id = ppc.product_category_id
         WHERE ppc.partner_company_id = $1 ORDER BY pc.name",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "categories": categories })))
}
