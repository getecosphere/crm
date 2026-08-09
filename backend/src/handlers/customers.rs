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
pub struct CustomerRow {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postcode: Option<String>,
    pub created_by_user_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CustomerListItem {
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub customer: CustomerRow,
    pub created_by_name: String,
    pub leads_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CustomerLead {
    pub id: Uuid,
    pub product_category_id: Uuid,
    pub category_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomerRequest {
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postcode: Option<String>,
    pub product_category_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub sales_rep_id: Option<Uuid>,
}

/// The primary workflow: one POST creates the Customer and one Lead per
/// selected Product Category inside a single transaction. If lead creation
/// fails the customer is rolled back too.
pub async fn create_customer(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<CreateCustomerRequest>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    user.require_role(&["ADMIN", "SALES_REP"])?;
    crate::error::require_non_blank(&[
        ("first_name", &req.first_name),
        ("last_name", &req.last_name),
        ("phone", &req.phone),
    ])?;
    if req.product_category_ids.is_empty() {
        return Err(AppError::BadRequest(
            "Select at least one product category for the lead".into(),
        ));
    }

    let mut tx = state.pool.begin().await?;

    let customer_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO customers (first_name, last_name, phone, email, address, city, postcode, created_by_user_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(req.first_name.trim())
    .bind(req.last_name.trim())
    .bind(req.phone.trim())
    .bind(req.email.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.address.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.city.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.postcode.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await?;

    let mut leads = Vec::new();
    for category_id in &req.product_category_ids {
        let exists: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM product_categories WHERE id = $1 AND status = 'active'")
                .bind(category_id)
                .fetch_optional(&mut *tx)
                .await?;
        if exists.is_none() {
            continue;
        }
        let lead_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO leads (customer_id, product_category_id, created_by_user_id) \
             VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(customer_id)
        .bind(category_id)
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await?;
        let category_name = sqlx::query_scalar::<_, String>(
            "SELECT name FROM product_categories WHERE id = $1",
        )
        .bind(category_id)
        .fetch_one(&mut *tx)
        .await?;
        leads.push(serde_json::json!({ "id": lead_id, "categoryName": category_name }));
    }

    tx.commit().await?;

    let customer_name = format!(
        "{} {}",
        req.first_name.trim(),
        req.last_name.trim()
    );
    let lead_count = leads.len();
    let admin_ids = crate::handlers::users::admin_auth_user_ids(&state).await;
    let _ = crate::notifications::push(
        &state.config,
        crate::notifications::IngestNotification {
            recipient_ids: admin_ids,
            kind: "customer_registered",
            title: "New customer registered",
            body: &format!(
                "{customer_name} was registered by {} with {lead_count} lead{}",
                user.name,
                if lead_count == 1 { "" } else { "s" }
            ),
            link: Some(&format!("/admin/customer/?id={customer_id}")),
            reference_id: Some(&customer_id.to_string()),
        },
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "customerId": customer_id,
            "leads": leads,
        })),
    ))
}

pub async fn list_customers(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<CustomerListItem>>> {
    let mut qb = sqlx::QueryBuilder::new(
        "SELECT
            c.id, c.first_name, c.last_name, c.phone, c.email, c.address, c.city, c.postcode,
            c.created_by_user_id, c.created_at,
            u.name AS created_by_name,
            (SELECT COUNT(*) FROM leads l WHERE l.customer_id = c.id) AS leads_count
         FROM customers c JOIN users u ON u.id = c.created_by_user_id WHERE 1=1",
    );
    if !user.is_admin() {
        qb.push(" AND c.created_by_user_id = ")
            .push_bind(user.id);
    }
    if let Some(sales_rep_id) = query.sales_rep_id {
        qb.push(" AND c.created_by_user_id = ").push_bind(sales_rep_id);
    }
    if let Some(search) = query.search.filter(|s| !s.trim().is_empty()) {
        let p1 = format!("%{search}%");
        let p2 = format!("%{search}%");
        let p3 = format!("%{search}%");
        let p4 = format!("%{search}%");
        qb.push(" AND (c.first_name ILIKE ")
            .push_bind(p1)
            .push(" OR c.last_name ILIKE ")
            .push_bind(p2)
            .push(" OR c.phone ILIKE ")
            .push_bind(p3)
            .push(" OR c.email ILIKE ")
            .push_bind(p4)
            .push(")");
    }
    qb.push(" ORDER BY c.created_at DESC");
    let rows = qb.build_query_as::<CustomerListItem>().fetch_all(&state.pool).await?;
    Ok(Json(rows))
}

pub async fn get_customer(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let customer = sqlx::query_as::<_, CustomerRow>(
        "SELECT id, first_name, last_name, phone, email, address, city, postcode, created_by_user_id, created_at \
         FROM customers WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Customer not found".into()))?;

    if !user.is_admin() && customer.created_by_user_id != user.id {
        return Err(AppError::Forbidden("Access denied".into()));
    }

    let created_by = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, name FROM users WHERE id = $1",
    )
    .bind(customer.created_by_user_id)
    .fetch_optional(&state.pool)
    .await?
    .map(|(id, name)| serde_json::json!({ "id": id, "name": name }));

    let leads = sqlx::query_as::<_, CustomerLead>(
        "SELECT l.id, l.product_category_id, pc.name AS category_name, l.created_at
         FROM leads l JOIN product_categories pc ON pc.id = l.product_category_id
         WHERE l.customer_id = $1 ORDER BY l.created_at DESC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "customer": customer,
        "createdBy": created_by,
        "leads": leads,
    })))
}
