use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{error::AppError, jwt, state::AppState};

/// The CRM business profile attached to an authenticated auth-domain account.
#[derive(Debug, Clone, FromRow)]
pub struct CrmUser {
    pub id: Uuid,
    pub auth_user_id: String,
    pub role: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub status: String,
    pub partner_company_id: Option<Uuid>,
}

/// Extractor that validates the shared JWT and loads the matching CRM user
/// profile. Authorization is always based on the CRM `users` row (the role
/// granted by the CRM administrator), never on the auth JWT `role` claim.
pub struct CurrentUser(pub CrmUser);

impl std::ops::Deref for CurrentUser {
    type Target = CrmUser;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Extractor that validates the shared JWT only — used by the setup claim
/// endpoint, which runs before any CRM user profile exists.
pub struct JwtUser {
    pub user_id: String,
    pub username: String,
}

pub struct AuthRejection(AppError);

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for JwtUser {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AuthRejection(AppError::Unauthorized("missing bearer token".into())))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AuthRejection(AppError::Unauthorized("missing bearer token".into())))?;

        let claims = jwt::validate_token(&state.config.jwt_secret, token)
            .ok_or_else(|| AuthRejection(AppError::Unauthorized("invalid or expired token".into())))?;

        Ok(JwtUser {
            user_id: claims.sub,
            username: claims.username,
        })
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AuthRejection(AppError::Unauthorized("missing bearer token".into())))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AuthRejection(AppError::Unauthorized("missing bearer token".into())))?;

        let claims = jwt::validate_token(&state.config.jwt_secret, token)
            .ok_or_else(|| AuthRejection(AppError::Unauthorized("invalid or expired token".into())))?;

        let user = sqlx::query_as::<_, CrmUser>(
            "SELECT id, auth_user_id, role, name, email, phone, status, partner_company_id \
             FROM users WHERE auth_user_id = $1 AND status = 'active'",
        )
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AuthRejection(AppError::Internal(anyhow::anyhow!(e))))?
        .ok_or_else(|| {
            AuthRejection(AppError::Forbidden(
                "This account is not provisioned in the CRM. Contact your administrator.".into(),
            ))
        })?;

        Ok(CurrentUser(user))
    }
}

impl CrmUser {
    /// Mirrors `@PreAuthorize("hasAnyRole(...)")`: 403 when the CRM role is
    /// not in the allowed set.
    pub fn require_role(&self, allowed: &[&str]) -> Result<(), AppError> {
        let role_upper = self.role.to_uppercase();
        if allowed.iter().any(|r| r.to_uppercase() == role_upper) {
            Ok(())
        } else {
            tracing::warn!(
                user_id = %self.id,
                role = %self.role,
                required = ?allowed,
                "access denied: role not permitted"
            );
            Err(AppError::Forbidden("Access denied".into()))
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role.eq_ignore_ascii_case("ADMIN")
    }

    pub fn is_sales_rep(&self) -> bool {
        self.role.eq_ignore_ascii_case("SALES_REP")
    }

    pub fn is_partner(&self) -> bool {
        self.role.eq_ignore_ascii_case("PARTNER")
    }
}

/// Builds a 401 response for endpoints that need any authenticated session.
pub fn unauthorized_response(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Unauthorized", "message": message })),
    )
        .into_response()
}
