use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::CurrentUser,
    error::{AppError, AppResult},
    state::AppState,
};

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SaleRow {
    pub id: Uuid,
    pub customer_name: String,
    pub category_name: String,
    pub partner_name: String,
    pub sale_date: chrono::NaiveDate,
    pub sale_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub registered_by_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_sales(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Json<Vec<SaleRow>>> {
    let mut qb = sqlx::QueryBuilder::new(
        "SELECT s.id, c.first_name || ' ' || c.last_name AS customer_name,
                pc.name AS category_name, p.name AS partner_name, s.sale_date, s.sale_value,
                s.reference, s.notes, ru.name AS registered_by_name, s.created_at
         FROM sales s
         JOIN lead_assignments la ON la.id = s.lead_assignment_id
         JOIN leads l ON l.id = la.lead_id
         JOIN customers c ON c.id = l.customer_id
         JOIN product_categories pc ON pc.id = l.product_category_id
         JOIN partner_companies p ON p.id = la.partner_company_id
         JOIN users ru ON ru.id = s.registered_by_user_id
         WHERE 1=1",
    );
    if user.is_partner() {
        let company = user
            .partner_company_id
            .ok_or_else(|| AppError::Forbidden("Partner user has no partner company".into()))?;
        qb.push(" AND la.partner_company_id = ").push_bind(company);
    } else if !user.is_admin() {
        qb.push(" AND l.created_by_user_id = ").push_bind(user.id);
    }
    qb.push(" ORDER BY s.created_at DESC");
    let rows = qb.build_query_as::<SaleRow>().fetch_all(&state.pool).await?;
    Ok(Json(rows))
}

pub async fn get_sale(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<SaleRow>> {
    let row = sqlx::query_as::<_, SaleRow>(
        "SELECT s.id, c.first_name || ' ' || c.last_name AS customer_name,
                pc.name AS category_name, p.name AS partner_name, s.sale_date, s.sale_value,
                s.reference, s.notes, ru.name AS registered_by_name, s.created_at
         FROM sales s
         JOIN lead_assignments la ON la.id = s.lead_assignment_id
         JOIN leads l ON l.id = la.lead_id
         JOIN customers c ON c.id = l.customer_id
         JOIN product_categories pc ON pc.id = l.product_category_id
         JOIN partner_companies p ON p.id = la.partner_company_id
         JOIN users ru ON ru.id = s.registered_by_user_id
         WHERE s.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Sale not found".into()))?;

    if user.is_partner() {
        let company = user
            .partner_company_id
            .ok_or_else(|| AppError::Forbidden("Partner user has no partner company".into()))?;
        let owned = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM sales s JOIN lead_assignments la ON la.id = s.lead_assignment_id
                WHERE s.id = $1 AND la.partner_company_id = $2
            )",
        )
        .bind(id)
        .bind(company)
        .fetch_one(&state.pool)
        .await?;
        if !owned {
            return Err(AppError::Forbidden("Access denied".into()));
        }
    } else if !user.is_admin() {
        return Err(AppError::Forbidden("Access denied".into()));
    }
    Ok(Json(row))
}
