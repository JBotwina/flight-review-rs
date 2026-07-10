pub mod ai;
pub mod api;
pub mod auth;
pub mod db;
pub mod extract;
pub mod geocode;
pub mod storage;

use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

/// Shared application state passed to all handlers via axum's State extractor.
pub struct AppState {
    pub db: Arc<dyn db::LogStore>,
    pub storage: Arc<storage::FileStorage>,
    /// Prefix where v1 .ulg files live in the same storage backend.
    /// E.g., `flight_review/log_files` for `s3://bucket/flight_review/log_files/<uuid>.ulg`.
    pub v1_ulg_prefix: Option<String>,
    /// Mapbox access token for reverse geocoding at upload time.
    pub mapbox_token: Option<String>,
    /// Shared HTTP client for outbound requests (geocoding, etc.).
    pub http_client: reqwest::Client,
    /// OpenRouter client. None when OPENROUTER_API_KEY is not configured.
    pub openrouter: Option<ai::OpenRouterClient>,
    /// Shared-password access control. None only for local development/tests.
    pub access_control: Option<auth::AccessControl>,
}

/// Build the application router. Shared by the binary (`main.rs`) and the
/// integration tests so they can never drift out of sync.
pub fn build_router(state: Arc<AppState>) -> Router {
    let protected = Router::new()
        .route("/api/version", get(api::version::version))
        .route("/api/ai/models", get(api::ai::list_models))
        .route("/api/ai/balance", get(api::ai::get_balance))
        .route(
            "/api/upload",
            post(api::upload::upload).layer(DefaultBodyLimit::max(512 * 1024 * 1024)), // 512 MB
        )
        .route("/api/logs", get(api::logs::list_logs))
        .route("/api/logs/facets", get(api::logs::list_facets))
        .route("/api/stats", get(api::stats::get_stats))
        .route(
            "/api/logs/{id}",
            get(api::logs::get_log)
                .patch(api::logs::update_log_metadata)
                .delete(api::logs::delete_log),
        )
        .route(
            "/api/logs/{id}/ai-analysis",
            get(api::ai::get_analysis).post(api::ai::generate_analysis),
        )
        .route("/api/logs/{id}/track", get(api::logs::get_track))
        .route(
            "/api/logs/{id}/data/{filename}",
            get(api::logs::get_log_file),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_access,
        ));

    Router::new()
        .route("/health", get(api::health::health))
        .route("/api/auth/session", get(auth::session))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .merge(protected)
        .with_state(state)
}
