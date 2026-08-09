use std::time::Duration;

use axum::{
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
};

use crate::{handlers, state::AppState};

const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

pub fn build_router(state: AppState) -> Router {
    let origins: Vec<_> = state
        .config
        .cors_allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(AllowMethods::list([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
            axum::http::Method::HEAD,
        ]))
        .allow_headers(AllowHeaders::mirror_request())
        .expose_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600));

    let api_routes = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/", get(handlers::health::root))
        .route("/setup/status", get(handlers::setup::setup_status))
        .route("/setup/claim", axum::routing::post(handlers::setup::setup_claim))
        .route("/users/me", get(handlers::users::me))
        .route(
            "/users",
            get(handlers::users::list_users).post(handlers::users::create_user),
        )
        .route(
            "/users/:id",
            get(handlers::users::get_user).put(handlers::users::update_user),
        )
        .route(
            "/partners",
            get(handlers::partners::list_partners).post(handlers::partners::create_partner),
        )
        .route(
            "/partners/:id",
            get(handlers::partners::get_partner).put(handlers::partners::update_partner),
        )
        .route(
            "/partners/:id/categories",
            axum::routing::put(handlers::partners::set_partner_categories),
        )
        .route(
            "/product-categories",
            get(handlers::product_categories::list_categories)
                .post(handlers::product_categories::create_category),
        )
        .route(
            "/product-categories/:id",
            get(handlers::product_categories::get_category)
                .put(handlers::product_categories::update_category),
        )
        .route(
            "/product-categories/:id/partners",
            axum::routing::put(handlers::product_categories::set_category_partners),
        )
        .route(
            "/customers",
            get(handlers::customers::list_customers).post(handlers::customers::create_customer),
        )
        .route("/customers/:id", get(handlers::customers::get_customer))
        .route(
            "/leads",
            get(handlers::leads::list_leads),
        )
        .route(
            "/leads/:id",
            get(handlers::leads::get_lead),
        )
        .route(
            "/leads/:id/assign",
            axum::routing::put(handlers::leads::assign_lead),
        )
        .route(
            "/lead-assignments",
            get(handlers::lead_assignments::list_assignments),
        )
        .route(
            "/lead-assignments/:id",
            get(handlers::lead_assignments::get_assignment),
        )
        .route(
            "/lead-assignments/:id/status",
            axum::routing::put(handlers::lead_assignments::update_status),
        )
        .route(
            "/lead-assignments/:id/sale",
            axum::routing::post(handlers::lead_assignments::register_sale),
        )
        .route(
            "/lead-assignments/:id/no-sale",
            axum::routing::post(handlers::lead_assignments::register_no_sale),
        )
        .route("/sales", get(handlers::sales::list_sales))
        .route("/sales/:id", get(handlers::sales::get_sale))
        .route("/dashboard", get(handlers::dashboard::dashboard))
        .route("/reports", get(handlers::reports::reports))
        .layer(
            tower::ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
                .layer(axum::middleware::map_response(security_headers)),
        )
        .with_state(state);

    Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .layer(axum::middleware::from_fn(crate::request_id::propagate))
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

async fn security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("x-xss-protection", HeaderValue::from_static("0"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "cache-control",
        HeaderValue::from_static("no-cache, no-store, max-age=0, must-revalidate"),
    );
    headers.insert("pragma", HeaderValue::from_static("no-cache"));
    response
}

pub fn rate_limit_response() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        axum::Json(serde_json::json!({ "error": "Too many requests" })),
    )
        .into_response()
}
