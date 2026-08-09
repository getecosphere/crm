use std::env;

const KNOWN_PLACEHOLDER_SECRETS: &[&str] = &[
    "your-secret-key-change-in-production",
    "change-this-secret",
    "secret",
];

const MIN_JWT_SECRET_BYTES: usize = 32;
const RECOMMENDED_JWT_SECRET_BYTES: usize = 64;

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_default();
        if jwt_secret.is_empty() {
            anyhow::bail!(
                "JWT_SECRET is not set. Refusing to start with no signing key -- \
                 generate one (e.g. `openssl rand -base64 48`) and set it in .env."
            );
        }
        if KNOWN_PLACEHOLDER_SECRETS.contains(&jwt_secret.as_str()) {
            anyhow::bail!(
                "JWT_SECRET is set to a known placeholder value. Refusing to start -- \
                 generate a real secret and set it in .env."
            );
        }
        if jwt_secret.len() < MIN_JWT_SECRET_BYTES {
            anyhow::bail!(
                "JWT_SECRET is only {} bytes; refusing to start with fewer than {} \
                 for HS512 signing.",
                jwt_secret.len(),
                MIN_JWT_SECRET_BYTES
            );
        }
        if jwt_secret.len() < RECOMMENDED_JWT_SECRET_BYTES {
            tracing::warn!(
                bytes = jwt_secret.len(),
                recommended = RECOMMENDED_JWT_SECRET_BYTES,
                "JWT_SECRET is shorter than recommended for HS512"
            );
        }

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://crm_user:crm@localhost:5432/crm".to_string()),
            jwt_secret,
            server_port,
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        })
    }
}
