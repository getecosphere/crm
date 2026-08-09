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
pub struct CategoryRow {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CategoryListItem {
    #[sqlx(flatten)]
    pub category: CategoryRow,
    pub partners_count: i64,
    pub leads_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PartnerShort {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerIdsRequest {
    pub partner_company_ids: Vec<Uuid>,
}

pub async fn list_categories(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Json<Vec<CategoryListItem>>> {
    let rows = sqlx::query_as::<_, CategoryListItem>(
        "SELECT
            pc.id, pc.name, pc.description, pc.status, pc.created_at,
            (SELECT COUNT(*) FROM partner_product_categories ppc WHERE ppc.product_category_id = pc.id) AS partners_count,
            (SELECT COUNT(*) FROM leads l WHERE l.product_category_id = pc.id) AS leads_count
         FROM product_categories pc
         WHERE pc.status = 'active'
         ORDER BY pc.name",
    )
    .fetch_all(&state.pool)
    .await?;
    let _ = user.id;
    Ok(Json(rows))
}

pub async fn create_category(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<CreateCategoryRequest>,
) -> AppResult<(StatusCode, Json<CategoryRow>)> {
    user.require_role(&["ADMIN"])?;
    crate::error::require_non_blank(&[("name", &req.name)])?;
    let row = sqlx::query_as::<_, CategoryRow>(
        "INSERT INTO product_categories (name, description, status) VALUES ($1, $2, 'active') \
         RETURNING id, name, description, status, created_at",
    )
    .bind(req.name.trim())
    .bind(req.description.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn update_category(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCategoryRequest>,
) -> AppResult<Json<CategoryRow>> {
    user.require_role(&["ADMIN"])?;
    if let Some(status) = &req.status {
        if !["active", "inactive"].contains(&status.as_str()) {
            return Err(AppError::BadRequest("Status must be active or inactive".into()));
        }
    }
    let row = sqlx::query_as::<_, CategoryRow>(
        "UPDATE product_categories SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            status = COALESCE($4, status),
            updated_at = now()
         WHERE id = $1
         RETURNING id, name, description, status, created_at",
    )
    .bind(id)
    .bind(req.name.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.description.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.status.as_deref().map(|s| s.to_lowercase()))
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Category not found".into()))?;
    Ok(Json(row))
}

pub async fn get_category(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_role(&["ADMIN"])?;
    let category = sqlx::query_as::<_, CategoryRow>(
        "SELECT id, name, description, status, created_at FROM product_categories WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Category not found".into()))?;

    let partners = sqlx::query_as::<_, PartnerShort>(
        "SELECT pc.id, pc.name FROM partner_product_categories ppc
         JOIN partner_companies pc ON pc.id = ppc.partner_company_id
         WHERE ppc.product_category_id = $1 AND pc.status = 'active' ORDER BY pc.name",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "category": category,
        "partners": partners,
    })))
}

pub async fn set_category_partners(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<PartnerIdsRequest>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_role(&["ADMIN"])?;
    let exists: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM product_categories WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;
    if exists.is_none() {
        return Err(AppError::NotFound("Category not found".into()));
    }

    let mut tx = state.pool.begin().await?;
    sqlx::query("DELETE FROM partner_product_categories WHERE product_category_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for partner_id in &req.partner_company_ids {
        let exists: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM partner_companies WHERE id = $1 AND status = 'active'")
                .bind(partner_id)
                .fetch_optional(&mut *tx)
                .await?;
        if let Some(_) = exists {
            sqlx::query(
                "INSERT INTO partner_product_categories (partner_company_id, product_category_id) \
                 VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(partner_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;

    let partners = sqlx::query_as::<_, PartnerShort>(
        "SELECT pc.id, pc.name FROM partner_product_categories ppc
         JOIN partner_companies pc ON pc.id = ppc.partner_company_id
         WHERE ppc.product_category_id = $1 ORDER BY pc.name",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "partners": partners })))
}
