use chrono::Utc;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

/// Mirrors the claim shape issued by the auth domain (HS512): `sub`,
/// `username`, `role`, `iat`, `exp`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub iat: i64,
    pub exp: i64,
}

pub fn validate_token(secret: &str, token: &str) -> Option<Claims> {
    let validation = Validation::new(Algorithm::HS512);
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .ok()
    .map(|data| data.claims)
}

pub fn now_timestamp() -> i64 {
    Utc::now().timestamp()
}
