// === AI Config Pricing Engine ===
// Hệ thống định giá thông minh cho NFT & Stock
// Sử dụng Rust + Tokio multi-threaded runtime

mod config;
mod engine;
mod scrapers;
mod crew_integration;
mod storage;
mod api;

use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // === 1. Khởi tạo logging ===
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "valuation_pricing_tools=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("=== AI Config Pricing Engine v0.2.0 ===");
    info!("Khởi động hệ thống...");

    // === 2. Load cấu hình ===
    let app_config = config::load_config("config/default")
        .unwrap_or_else(|e| {
            info!("Không load được config file: {}. Dùng config mặc định.", e);
            config::AppConfig::default()
        });

    let server_host = app_config.server.host.clone();
    let server_port = app_config.server.port;
    let db_url = app_config.database.url.clone();

    // === 3. Khởi tạo Config Manager (ArcSwap hot-reload) ===
    let config_manager = Arc::new(config::DynamicConfigManager::new(app_config));
    info!("Config Manager đã sẵn sàng (hot-reload enabled)");

    // === 4. Khởi tạo Database ===
    let db_pool = storage::init_database(&db_url).await?;
    info!("Database đã kết nối");

    // === 5. Tạo shared state ===
    let state = Arc::new(api::AppState {
        config_manager,
        db_pool,
    });

    // === 6. Tạo router ===
    let router = api::create_router(state);

    // === 7. Khởi động server ===
    let addr = format!("{}:{}", server_host, server_port);
    info!("Server đang lắng nghe tại: http://{}", addr);
    info!("API docs: http://{}/api/v1/health", addr);
    info!("---");
    info!("Endpoints:");
    info!("  GET  /api/v1/health              - Health check");
    info!("  GET  /api/v1/config              - Lấy cấu hình");
    info!("  PUT  /api/v1/config              - Cập nhật cấu hình");
    info!("  GET  /api/v1/config/attributes   - Lấy dynamic attributes");
    info!("  PUT  /api/v1/config/attributes   - Cập nhật attributes");
    info!("  PUT  /api/v1/config/weights/nft  - Cập nhật trọng số NFT");
    info!("  PUT  /api/v1/config/weights/stock - Cập nhật trọng số Stock");
    info!("  POST /api/v1/valuate/nft         - Định giá NFT");
    info!("  POST /api/v1/valuate/stock       - Phân tích cổ phiếu (technical)");
    info!("  POST /api/v1/valuate/stock/fundamental - Phân tích cổ phiếu (fundamental)");
    info!("  POST /api/v1/valuate/batch       - Batch valuation");
    info!("  GET  /api/v1/history/:asset_id   - Lịch sử định giá");
    info!("  POST /api/v1/crew/webhook        - CrewAI webhook");
    info!("---");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
