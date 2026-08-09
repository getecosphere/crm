use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::FromRow;

use crate::{
    auth::CurrentUser,
    error::AppResult,
    state::AppState,
};

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ReportRow {
    pub label: String,
    pub value: i64,
}

pub async fn reports(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Json<serde_json::Value>> {
    user.require_role(&["ADMIN"])?;

    let customers_by_rep = sqlx::query_as::<_, ReportRow>(
        "SELECT u.name AS label, COUNT(c.id)::bigint AS value
         FROM users u LEFT JOIN customers c ON c.created_by_user_id = u.id
         WHERE u.role = 'SALES_REP' GROUP BY u.name ORDER BY value DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    let leads_by_rep = sqlx::query_as::<_, ReportRow>(
        "SELECT u.name AS label, COUNT(l.id)::bigint AS value
         FROM users u LEFT JOIN leads l ON l.created_by_user_id = u.id
         WHERE u.role = 'SALES_REP' GROUP BY u.name ORDER BY value DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    let leads_by_category = sqlx::query_as::<_, ReportRow>(
        "SELECT pc.name AS label, COUNT(l.id)::bigint AS value
         FROM product_categories pc LEFT JOIN leads l ON l.product_category_id = pc.id
         GROUP BY pc.name ORDER BY value DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    let leads_by_partner = sqlx::query_as::<_, ReportRow>(
        "SELECT p.name AS label, COUNT(la.id)::bigint AS value
         FROM partner_companies p LEFT JOIN lead_assignments la ON la.partner_company_id = p.id
         GROUP BY p.name ORDER BY value DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    let sales_by_partner = sqlx::query_as::<_, ReportRow>(
        "SELECT p.name AS label, COUNT(s.id)::bigint AS value
         FROM partner_companies p
         LEFT JOIN lead_assignments la ON la.partner_company_id = p.id
         LEFT JOIN sales s ON s.lead_assignment_id = la.id
         GROUP BY p.name ORDER BY value DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    let status_by_partner = sqlx::query_as::<_, ReportRow>(
        "SELECT p.name || ' — ' || la.status AS label, COUNT(*)::bigint AS value
         FROM lead_assignments la JOIN partner_companies p ON p.id = la.partner_company_id
         GROUP BY p.name, la.status ORDER BY p.name, la.status",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "customersByRep": customers_by_rep,
        "leadsByRep": leads_by_rep,
        "leadsByCategory": leads_by_category,
        "leadsByPartner": leads_by_partner,
        "salesByPartner": sales_by_partner,
        "statusByPartner": status_by_partner,
    })))
}
