use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

use crate::{
    auth::{CrmUser, CurrentUser},
    error::{AppError, AppResult},
    state::AppState,
};

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentListItem {
    pub id: Uuid,
    pub lead_id: Uuid,
    pub customer_name: String,
    pub category_name: String,
    pub partner_name: String,
    pub status: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub no_sale_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterSaleRequest {
    pub sale_date: Option<chrono::NaiveDate>,
    pub sale_value: Option<f64>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoSaleRequest {
    pub reason: Option<String>,
    pub notes: Option<String>,
}

const PROCESSING_STATUSES: &[&str] = &["NEW", "CONTACTED", "IN_PROGRESS"];

async fn load_and_authorize(
    state: &AppState,
    user: &CrmUser,
    assignment_id: Uuid,
) -> Result<(Uuid, Uuid, Uuid), AppError> {
    let assignment = sqlx::query_as::<_, (Uuid, Uuid, Uuid)>(
        "SELECT id, partner_company_id, lead_id FROM lead_assignments WHERE id = $1",
    )
    .bind(assignment_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Assignment not found".into()))?;

    if user.is_partner() {
        let company = user
            .partner_company_id
            .ok_or_else(|| AppError::Forbidden("Partner user has no partner company".into()))?;
        if assignment.1 != company {
            return Err(AppError::Forbidden("Access denied".into()));
        }
    } else if !user.is_admin() {
        return Err(AppError::Forbidden("Access denied".into()));
    }
    Ok(assignment)
}

/// Notifies the sales rep who created the lead, plus every admin, about a
/// partner action on an assignment (status change, sale, no-sale).
async fn notify_lead_progress(
    state: &AppState,
    lead_id: Uuid,
    partner_name: &str,
    title: &str,
    body: String,
) {
    let _ = partner_name;
    let mut recipient_ids =
        crate::handlers::users::admin_auth_user_ids(state).await;
    if let Some(creator) = crate::handlers::users::lead_creator_auth_id(state, lead_id).await {
        if !recipient_ids.contains(&creator) {
            recipient_ids.push(creator);
        }
    }
    if recipient_ids.is_empty() {
        return;
    }
    let _ = crate::notifications::push(
        &state.config,
        crate::notifications::IngestNotification {
            recipient_ids,
            kind: "assignment_progress",
            title,
            body: &body,
            link: Some(&format!("/admin/lead/?id={lead_id}")),
            reference_id: Some(&lead_id.to_string()),
        },
    )
    .await;
}

pub async fn list_assignments(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<AssignmentListItem>>> {
    let mut qb = sqlx::QueryBuilder::new(
        "SELECT la.id, la.lead_id, c.first_name || ' ' || c.last_name AS customer_name,
                pc.name AS category_name, p.name AS partner_name, la.status, la.assigned_at, la.no_sale_reason
         FROM lead_assignments la
         JOIN leads l ON l.id = la.lead_id
         JOIN customers c ON c.id = l.customer_id
         JOIN product_categories pc ON pc.id = l.product_category_id
         JOIN partner_companies p ON p.id = la.partner_company_id
         WHERE 1=1",
    );
    if user.is_partner() {
        let company = user
            .partner_company_id
            .ok_or_else(|| AppError::Forbidden("Partner user has no partner company".into()))?;
        qb.push(" AND la.partner_company_id = ").push_bind(company);
    } else if !user.is_admin() {
        return Err(AppError::Forbidden("Access denied".into()));
    }
    if let Some(status) = query.status.filter(|s| !s.trim().is_empty()) {
        qb.push(" AND la.status = ").push_bind(status.to_uppercase());
    }
    qb.push(" ORDER BY la.assigned_at DESC");
    let rows = qb
        .build_query_as::<AssignmentListItem>()
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

pub async fn get_assignment(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let (_, _, _) = load_and_authorize(&state, &user, id).await?;

    let assignment = sqlx::query(
        "SELECT jsonb_build_object(
            'id', la.id, 'leadId', la.lead_id, 'partnerCompanyId', la.partner_company_id,
            'partnerName', p.name, 'status', la.status, 'assignedAt', la.assigned_at,
            'noSaleReason', la.no_sale_reason, 'noSaleNotes', la.no_sale_notes
         ) FROM lead_assignments la JOIN partner_companies p ON p.id = la.partner_company_id
         WHERE la.id = $1",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?
    .get::<serde_json::Value, _>(0);

    let customer = sqlx::query(
        "SELECT jsonb_build_object(
            'id', c.id, 'firstName', c.first_name, 'lastName', c.last_name, 'phone', c.phone,
            'email', c.email, 'address', c.address, 'city', c.city, 'postcode', c.postcode,
            'createdAt', c.created_at, 'createdBy', u.name
         )
         FROM lead_assignments la
         JOIN leads l ON l.id = la.lead_id
         JOIN customers c ON c.id = l.customer_id
         JOIN users u ON u.id = c.created_by_user_id
         WHERE la.id = $1",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?
    .get::<serde_json::Value, _>(0);

    let lead = sqlx::query(
        "SELECT jsonb_build_object(
            'id', l.id, 'categoryId', pc.id, 'categoryName', pc.name,
            'createdBy', u.name, 'createdAt', l.created_at
         )
         FROM lead_assignments la
         JOIN leads l ON l.id = la.lead_id
         JOIN product_categories pc ON pc.id = l.product_category_id
         JOIN users u ON u.id = l.created_by_user_id
         WHERE la.id = $1",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?
    .get::<serde_json::Value, _>(0);

    let sale = sqlx::query(
        "SELECT jsonb_build_object(
            'id', s.id, 'saleDate', s.sale_date, 'saleValue', s.sale_value,
            'reference', s.reference, 'notes', s.notes, 'registeredBy', u.name, 'createdAt', s.created_at
         )
         FROM sales s LEFT JOIN users u ON u.id = s.registered_by_user_id
         WHERE s.lead_assignment_id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .map(|row| row.get::<serde_json::Value, _>(0));

    Ok(Json(serde_json::json!({
        "assignment": assignment,
        "customer": customer,
        "lead": lead,
        "sale": sale,
    })))
}

pub async fn update_status(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStatusRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let status = req.status.to_uppercase();
    if !PROCESSING_STATUSES.contains(&status.as_str()) {
        return Err(AppError::BadRequest(
            "Use the dedicated sale / no-sale endpoints for those outcomes".into(),
        ));
    }
    let (_assignment_id, _partner_id, lead_id) = load_and_authorize(&state, &user, id).await?;

    let updated = sqlx::query(
        "UPDATE lead_assignments SET status = $2, no_sale_reason = NULL, no_sale_notes = NULL, updated_at = now()
         WHERE id = $1
         RETURNING jsonb_build_object('id', id, 'status', status)",
    )
    .bind(id)
    .bind(&status)
    .fetch_one(&state.pool)
    .await?
    .get::<serde_json::Value, _>(0);

    let status_label = status
        .split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    notify_lead_progress(
        &state,
        lead_id,
        &user.name,
        "Lead status updated",
        format!("{status_label} — {}", user.name),
    )
    .await;

    Ok(Json(updated))
}

pub async fn register_sale(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<RegisterSaleRequest>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let (_assignment_id, _partner_id, lead_id) = load_and_authorize(&state, &user, id).await?;

    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "UPDATE lead_assignments SET status = 'SALE', no_sale_reason = NULL, no_sale_notes = NULL, updated_at = now() WHERE id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    let sale_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO sales (lead_assignment_id, sale_date, sale_value, reference, notes, registered_by_user_id) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(id)
    .bind(req.sale_date.unwrap_or_else(|| chrono::Utc::now().date_naive()))
    .bind(req.sale_value)
    .bind(req.reference.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.notes.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    notify_lead_progress(
        &state,
        lead_id,
        &user.name,
        "Sale registered",
        format!("A sale was registered by {}", user.name),
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": sale_id, "status": "SALE" })),
    ))
}

pub async fn register_no_sale(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<NoSaleRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let (_assignment_id, _partner_id, lead_id) = load_and_authorize(&state, &user, id).await?;

    let updated = sqlx::query(
        "UPDATE lead_assignments SET status = 'NO_SALE', no_sale_reason = $2, no_sale_notes = $3, updated_at = now()
         WHERE id = $1
         RETURNING jsonb_build_object('id', id, 'status', status, 'noSaleReason', no_sale_reason)",
    )
    .bind(id)
    .bind(req.reason.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.notes.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .fetch_one(&state.pool)
    .await?
    .get::<serde_json::Value, _>(0);

    let reason = req
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("No sale");
    notify_lead_progress(
        &state,
        lead_id,
        &user.name,
        "No-sale recorded",
        format!("{reason} — recorded by {}", user.name),
    )
    .await;

    Ok(Json(updated))
}
