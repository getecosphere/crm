use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::FromRow;

use crate::{
    auth::{CrmUser, CurrentUser},
    error::{AppError, AppResult},
    state::AppState,
};

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CountByName {
    pub id: uuid::Uuid,
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecentLead {
    pub id: uuid::Uuid,
    pub customer_name: String,
    pub category_name: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PartnerRecentLead {
    pub id: uuid::Uuid,
    pub customer_name: String,
    pub category_name: String,
    pub status: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}

pub async fn dashboard(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Json<serde_json::Value>> {
    match user.role.to_uppercase().as_str() {
        "ADMIN" => admin_dashboard(&state).await,
        "SALES_REP" => sales_dashboard(&state, &user).await,
        "PARTNER" => partner_dashboard(&state, &user).await,
        _ => Err(AppError::Forbidden("Unknown role".into())),
    }
}

async fn admin_dashboard(state: &AppState) -> AppResult<Json<serde_json::Value>> {
    let total_customers = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM customers")
        .fetch_one(&state.pool)
        .await?;
    let total_leads = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM leads")
        .fetch_one(&state.pool)
        .await?;
    let total_sales = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sales")
        .fetch_one(&state.pool)
        .await?;
    let total_sales_reps = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE role = 'SALES_REP' AND status = 'active'",
    )
    .fetch_one(&state.pool)
    .await?;
    let total_partners = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM partner_companies WHERE status = 'active'",
    )
    .fetch_one(&state.pool)
    .await?;
    let unassigned_leads = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM leads l WHERE NOT EXISTS (SELECT 1 FROM lead_assignments la WHERE la.lead_id = l.id)",
    )
    .fetch_one(&state.pool)
    .await?;

    let leads_per_sales_rep = sqlx::query_as::<_, CountByName>(
        "SELECT u.id, u.name, COUNT(l.id)::bigint AS count
         FROM users u LEFT JOIN leads l ON l.created_by_user_id = u.id
         WHERE u.role = 'SALES_REP'
         GROUP BY u.id, u.name ORDER BY count DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    let leads_per_partner = sqlx::query_as::<_, CountByName>(
        "SELECT p.id, p.name, COUNT(la.id)::bigint AS count
         FROM partner_companies p LEFT JOIN lead_assignments la ON la.partner_company_id = p.id
         GROUP BY p.id, p.name ORDER BY count DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    let sales_per_partner = sqlx::query_as::<_, CountByName>(
        "SELECT p.id, p.name, COUNT(s.id)::bigint AS count
         FROM partner_companies p
         LEFT JOIN lead_assignments la ON la.partner_company_id = p.id
         LEFT JOIN sales s ON s.lead_assignment_id = la.id
         GROUP BY p.id, p.name ORDER BY count DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    let recent_leads = sqlx::query_as::<_, RecentLead>(
        "SELECT l.id, c.first_name || ' ' || c.last_name AS customer_name,
                pc.name AS category_name,
                COALESCE((SELECT la2.status FROM lead_assignments la2 WHERE la2.lead_id = l.id ORDER BY la2.assigned_at DESC LIMIT 1), 'UNASSIGNED') AS status,
                l.created_at
         FROM leads l
         JOIN customers c ON c.id = l.customer_id
         JOIN product_categories pc ON pc.id = l.product_category_id
         ORDER BY l.created_at DESC LIMIT 8",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "totalCustomers": total_customers,
        "totalLeads": total_leads,
        "totalSales": total_sales,
        "totalSalesReps": total_sales_reps,
        "totalPartners": total_partners,
        "unassignedLeads": unassigned_leads,
        "leadsPerSalesRep": leads_per_sales_rep,
        "leadsPerPartner": leads_per_partner,
        "salesPerPartner": sales_per_partner,
        "recentLeads": recent_leads,
    })))
}

async fn sales_dashboard(
    state: &AppState,
    user: &CrmUser,
) -> AppResult<Json<serde_json::Value>> {
    let my_customers = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM customers WHERE created_by_user_id = $1",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let my_leads = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM leads WHERE created_by_user_id = $1",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let my_sales = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sales s JOIN lead_assignments la ON la.id = s.lead_assignment_id
         JOIN leads l ON l.id = la.lead_id WHERE l.created_by_user_id = $1",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let my_unassigned = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM leads l WHERE l.created_by_user_id = $1
         AND NOT EXISTS (SELECT 1 FROM lead_assignments la WHERE la.lead_id = l.id)",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;

    let recent_leads = sqlx::query_as::<_, RecentLead>(
        "SELECT l.id, c.first_name || ' ' || c.last_name AS customer_name,
                pc.name AS category_name,
                COALESCE((SELECT la2.status FROM lead_assignments la2 WHERE la2.lead_id = l.id ORDER BY la2.assigned_at DESC LIMIT 1), 'UNASSIGNED') AS status,
                l.created_at
         FROM leads l
         JOIN customers c ON c.id = l.customer_id
         JOIN product_categories pc ON pc.id = l.product_category_id
         WHERE l.created_by_user_id = $1
         ORDER BY l.created_at DESC LIMIT 8",
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "myCustomers": my_customers,
        "myLeads": my_leads,
        "mySales": my_sales,
        "myUnassignedLeads": my_unassigned,
        "recentLeads": recent_leads,
    })))
}

async fn partner_dashboard(
    state: &AppState,
    user: &CrmUser,
) -> AppResult<Json<serde_json::Value>> {
    let company = user
        .partner_company_id
        .ok_or_else(|| AppError::Forbidden("Partner user has no partner company".into()))?;

    let assigned = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM lead_assignments WHERE partner_company_id = $1",
    )
    .bind(company)
    .fetch_one(&state.pool)
    .await?;
    let by_status = sqlx::query_as::<_, (String, i64)>(
        "SELECT status, COUNT(*)::bigint FROM lead_assignments
         WHERE partner_company_id = $1 GROUP BY status",
    )
    .bind(company)
    .fetch_all(&state.pool)
    .await?;

    let status_counts: serde_json::Map<String, serde_json::Value> = by_status
        .into_iter()
        .map(|(status, count)| (status, serde_json::json!(count)))
        .collect();

    let recent_leads = sqlx::query_as::<_, PartnerRecentLead>(
        "SELECT la.id, c.first_name || ' ' || c.last_name AS customer_name,
                pc.name AS category_name, la.status, la.assigned_at
         FROM lead_assignments la
         JOIN leads l ON l.id = la.lead_id
         JOIN customers c ON c.id = l.customer_id
         JOIN product_categories pc ON pc.id = l.product_category_id
         WHERE la.partner_company_id = $1
         ORDER BY la.assigned_at DESC LIMIT 8",
    )
    .bind(company)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "assignedLeads": assigned,
        "statusCounts": status_counts,
        "recentLeads": recent_leads,
    })))
}
