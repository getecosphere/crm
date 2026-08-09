use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

use crate::{
    auth::CurrentUser,
    error::{AppError, AppResult},
    state::AppState,
};

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LeadListItem {
    pub id: Uuid,
    pub customer_name: String,
    pub customer_id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub sales_rep_id: Uuid,
    pub sales_rep_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub assignments: serde_json::Value,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentRow {
    pub id: Uuid,
    pub lead_id: Uuid,
    pub partner_company_id: Uuid,
    pub partner_name: String,
    pub status: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub category_id: Option<Uuid>,
    pub sales_rep_id: Option<Uuid>,
    pub partner_id: Option<Uuid>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignRequest {
    pub partner_company_ids: Vec<Uuid>,
}

const LEAD_LIST_SELECT: &str = "
    SELECT
        l.id,
        c.first_name || ' ' || c.last_name AS customer_name,
        c.id AS customer_id,
        pc.id AS category_id,
        pc.name AS category_name,
        u.id AS sales_rep_id,
        u.name AS sales_rep_name,
        l.created_at,
        COALESCE(jsonb_agg(jsonb_build_object(
            'assignmentId', la.id,
            'partnerId', la.partner_company_id,
            'partnerName', p.name,
            'status', la.status
        )) FILTER (WHERE la.id IS NOT NULL), '[]'::jsonb) AS assignments
    FROM leads l
    JOIN customers c ON c.id = l.customer_id
    JOIN product_categories pc ON pc.id = l.product_category_id
    JOIN users u ON u.id = l.created_by_user_id
    LEFT JOIN lead_assignments la ON la.lead_id = l.id
    LEFT JOIN partner_companies p ON p.id = la.partner_company_id
    WHERE 1=1
";

pub async fn list_leads(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<LeadListItem>>> {
    let mut qb = sqlx::QueryBuilder::new(LEAD_LIST_SELECT);
    if user.is_partner() {
        let partner_id = user
            .partner_company_id
            .ok_or_else(|| AppError::Forbidden("Partner user has no partner company".into()))?;
        qb.push(" AND la.partner_company_id = ").push_bind(partner_id);
    } else if !user.is_admin() {
        qb.push(" AND l.created_by_user_id = ").push_bind(user.id);
    }
    if let Some(status) = query.status.filter(|s| !s.trim().is_empty()) {
        qb.push(" AND EXISTS (SELECT 1 FROM lead_assignments la2 WHERE la2.lead_id = l.id AND la2.status = ")
            .push_bind(status.to_uppercase())
            .push(")");
    }
    if let Some(category_id) = query.category_id {
        qb.push(" AND pc.id = ").push_bind(category_id);
    }
    if let Some(sales_rep_id) = query.sales_rep_id {
        qb.push(" AND l.created_by_user_id = ").push_bind(sales_rep_id);
    }
    if let Some(partner_id) = query.partner_id {
        qb.push(" AND EXISTS (SELECT 1 FROM lead_assignments la3 WHERE la3.lead_id = l.id AND la3.partner_company_id = ")
            .push_bind(partner_id)
            .push(")");
    }
    if let Some(search) = query.search.filter(|s| !s.trim().is_empty()) {
        let p1 = format!("%{search}%");
        let p2 = format!("%{search}%");
        let p3 = format!("%{search}%");
        qb.push(" AND (c.first_name ILIKE ")
            .push_bind(p1)
            .push(" OR c.last_name ILIKE ")
            .push_bind(p2)
            .push(" OR c.phone ILIKE ")
            .push_bind(p3)
            .push(")");
    }
    qb.push(" GROUP BY l.id, c.first_name, c.last_name, c.id, pc.id, pc.name, u.id, u.name, l.created_at");
    qb.push(" ORDER BY l.created_at DESC");
    let rows = qb.build_query_as::<LeadListItem>().fetch_all(&state.pool).await?;
    Ok(Json(rows))
}

pub async fn get_lead(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let lead = sqlx::query_as::<_, LeadListItem>(
        &format!("{LEAD_LIST_SELECT} AND l.id = $1 GROUP BY l.id, c.first_name, c.last_name, c.id, pc.id, pc.name, u.id, u.name, l.created_at"),
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Lead not found".into()))?;

    if user.is_partner() {
        let partner_id = user
            .partner_company_id
            .ok_or_else(|| AppError::Forbidden("Partner user has no partner company".into()))?;
        let has = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM lead_assignments WHERE lead_id = $1 AND partner_company_id = $2)",
        )
        .bind(id)
        .bind(partner_id)
        .fetch_one(&state.pool)
        .await?;
        if !has {
            return Err(AppError::Forbidden("Access denied".into()));
        }
    } else if !user.is_admin() && lead.sales_rep_id != user.id {
        return Err(AppError::Forbidden("Access denied".into()));
    }

    let customer = sqlx::query(
        "SELECT jsonb_build_object(
            'id', c.id, 'firstName', c.first_name, 'lastName', c.last_name, 'phone', c.phone,
            'email', c.email, 'address', c.address, 'city', c.city, 'postcode', c.postcode,
            'createdAt', c.created_at
         ) FROM customers c WHERE c.id = $1",
    )
    .bind(lead.customer_id)
    .fetch_one(&state.pool)
    .await?
    .get::<serde_json::Value, _>(0);

    let category = sqlx::query(
        "SELECT jsonb_build_object('id', pc.id, 'name', pc.name, 'description', pc.description)
         FROM product_categories pc WHERE pc.id = $1",
    )
    .bind(lead.category_id)
    .fetch_one(&state.pool)
    .await?
    .get::<serde_json::Value, _>(0);

    let eligible_partners = if user.is_admin() {
        sqlx::query(
            "SELECT COALESCE(jsonb_agg(jsonb_build_object('id', p.id, 'name', p.name) ORDER BY p.name), '[]'::jsonb)
             FROM partner_product_categories ppc JOIN partner_companies p ON p.id = ppc.partner_company_id
             WHERE ppc.product_category_id = $1 AND p.status = 'active'",
        )
        .bind(lead.category_id)
        .fetch_one(&state.pool)
        .await?
        .get::<serde_json::Value, _>(0)
    } else {
        serde_json::Value::Array(vec![])
    };

    Ok(Json(serde_json::json!({
        "lead": lead,
        "customer": customer,
        "category": category,
        "eligiblePartners": eligible_partners,
    })))
}

/// Assigns a lead to one or more eligible partner companies. Additive: already
/// assigned partners are kept, never duplicated.
pub async fn assign_lead(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AssignRequest>,
) -> AppResult<Json<Vec<AssignmentRow>>> {
    user.require_role(&["ADMIN"])?;
    if req.partner_company_ids.is_empty() {
        return Err(AppError::BadRequest(
            "Select at least one partner company".into(),
        ));
    }

    let lead = sqlx::query_as::<_, (Uuid, Uuid)>(
        "SELECT id, product_category_id FROM leads WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Lead not found".into()))?;

    let mut tx = state.pool.begin().await?;
    for partner_id in &req.partner_company_ids {
        let eligible = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM partner_product_categories
                WHERE partner_company_id = $1 AND product_category_id = $2
            )",
        )
        .bind(partner_id)
        .bind(lead.1)
        .fetch_one(&mut *tx)
        .await?;
        if eligible {
            sqlx::query(
                "INSERT INTO lead_assignments (lead_id, partner_company_id, assigned_by_user_id, status) \
                 VALUES ($1, $2, $3, 'NEW') ON CONFLICT (lead_id, partner_company_id) DO NOTHING",
            )
            .bind(id)
            .bind(partner_id)
            .bind(user.id)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;

    let assignments = sqlx::query_as::<_, AssignmentRow>(
        "SELECT la.id, la.lead_id, la.partner_company_id, p.name AS partner_name, la.status, la.assigned_at
         FROM lead_assignments la JOIN partner_companies p ON p.id = la.partner_company_id
         WHERE la.lead_id = $1 ORDER BY la.assigned_at",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(assignments))
}
