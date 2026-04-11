// === Storage Layer - Lưu trữ dữ liệu với SQLite ===
// Repository pattern, migrations, data access

pub mod models;

use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tracing::info;

/// Khởi tạo database connection pool
pub async fn init_database(database_url: &str) -> Result<SqlitePool> {
    info!("Kết nối database: {}", database_url);

    // Tạo thư mục data nếu chưa có
    if let Some(path) = database_url.strip_prefix("sqlite://") {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Chạy migrations
    run_migrations(&pool).await?;

    info!("Database đã sẵn sàng");
    Ok(pool)
}

/// Chạy migrations tạo tables
async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    info!("Chạy database migrations...");

    sqlx::query(
        r#"
        -- Bảng lưu lịch sử định giá
        CREATE TABLE IF NOT EXISTS valuation_history (
            id TEXT PRIMARY KEY,
            asset_type TEXT NOT NULL,
            asset_identifier TEXT NOT NULL,
            source TEXT NOT NULL,
            estimated_price REAL NOT NULL,
            currency TEXT NOT NULL DEFAULT 'USD',
            composite_score REAL NOT NULL DEFAULT 0.0,
            trend TEXT NOT NULL DEFAULT 'neutral',
            confidence_pct REAL NOT NULL DEFAULT 0.0,
            attribute_scores TEXT, -- JSON
            raw_data TEXT, -- JSON
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        -- Bảng lưu cấu hình snapshots
        CREATE TABLE IF NOT EXISTS config_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            config_data TEXT NOT NULL, -- JSON
            description TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        -- Bảng lưu dữ liệu giá collection/stock
        CREATE TABLE IF NOT EXISTS price_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            asset_type TEXT NOT NULL,
            identifier TEXT NOT NULL,
            price REAL NOT NULL,
            volume REAL,
            currency TEXT NOT NULL DEFAULT 'USD',
            source TEXT NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        -- Bảng lưu CrewAI webhook data
        CREATE TABLE IF NOT EXISTS crew_webhook_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            data_type TEXT NOT NULL,
            payload TEXT NOT NULL, -- JSON
            processed INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Tạo indexes
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_valuation_asset
        ON valuation_history(asset_type, asset_identifier);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_price_data_lookup
        ON price_data(asset_type, identifier, timestamp);
        "#,
    )
    .execute(pool)
    .await?;

    info!("Migrations hoàn tất");
    Ok(())
}
