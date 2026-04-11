// === API Layer - REST API với Axum ===
// Endpoints cho config management, valuation, history

pub mod handlers;
pub mod models;

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, put},
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use sqlx::SqlitePool;

use crate::config::DynamicConfigManager;

/// Shared application state
pub struct AppState {
    pub config_manager: Arc<DynamicConfigManager>,
    pub db_pool: SqlitePool,
}

/// Tạo router với tất cả endpoints
pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // === Config endpoints ===
        .route("/api/v1/config", get(handlers::get_config))
        .route("/api/v1/config", put(handlers::update_config))
        .route("/api/v1/config/attributes", get(handlers::get_attributes))
        .route("/api/v1/config/attributes", put(handlers::update_attributes))
        .route("/api/v1/config/weights/nft", put(handlers::update_nft_weights))
        .route("/api/v1/config/weights/stock", put(handlers::update_stock_weights))
        // === Valuation endpoints ===
        .route("/api/v1/valuate/nft", post(handlers::valuate_nft))
        .route("/api/v1/valuate/stock", post(handlers::valuate_stock))
        .route("/api/v1/valuate/stock/fundamental", post(handlers::valuate_stock_fundamental))
        .route("/api/v1/valuate/batch", post(handlers::valuate_batch))
        // === History endpoints ===
        .route("/api/v1/history/:asset_id", get(handlers::get_history))
        // === CrewAI webhook ===
        .route("/api/v1/crew/webhook", post(handlers::crew_webhook))
        // === Health check ===
        .route("/api/v1/health", get(handlers::health_check))
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
