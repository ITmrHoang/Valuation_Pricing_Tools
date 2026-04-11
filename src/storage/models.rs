// === Storage Models - Repository pattern cho data access ===

use anyhow::Result;
use sqlx::SqlitePool;
use tracing::info;

use crate::engine::ValuationResult;

/// Repository cho valuation history
pub struct ValuationRepository;

impl ValuationRepository {
    /// Lưu kết quả định giá vào database
    pub async fn save(pool: &SqlitePool, result: &ValuationResult) -> Result<()> {
        let attribute_scores_json = serde_json::to_string(&result.attribute_scores)?;
        let raw_data_json = result.raw_data.as_ref()
            .map(|d| serde_json::to_string(d).unwrap_or_default());
        let trend = format!("{:?}", result.trend);

        sqlx::query(
            r#"
            INSERT INTO valuation_history
            (id, asset_type, asset_identifier, source, estimated_price, currency,
             composite_score, trend, confidence_pct, attribute_scores, raw_data)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&result.id)
        .bind(format!("{:?}", result.asset_type))
        .bind(&result.asset_identifier)
        .bind(&result.source)
        .bind(result.estimated_price)
        .bind(&result.currency)
        .bind(result.composite_score)
        .bind(&trend)
        .bind(result.confidence_pct)
        .bind(&attribute_scores_json)
        .bind(&raw_data_json)
        .execute(pool)
        .await?;

        info!("Đã lưu kết quả định giá: {} ({})", result.asset_identifier, result.id);
        Ok(())
    }

    /// Lấy lịch sử định giá theo asset identifier
    pub async fn get_history(
        pool: &SqlitePool,
        asset_identifier: &str,
        limit: i32,
    ) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_as::<_, (String, String, String, f64, String, f64, String, f64, String,)>(
            r#"
            SELECT id, asset_type, asset_identifier, estimated_price, currency,
                   composite_score, trend, confidence_pct, created_at
            FROM valuation_history
            WHERE asset_identifier = ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(asset_identifier)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let results: Vec<serde_json::Value> = rows.iter().map(|row| {
            serde_json::json!({
                "id": row.0,
                "asset_type": row.1,
                "asset_identifier": row.2,
                "estimated_price": row.3,
                "currency": row.4,
                "composite_score": row.5,
                "trend": row.6,
                "confidence_pct": row.7,
                "created_at": row.8,
            })
        }).collect();

        Ok(results)
    }

    /// Lưu snapshot cấu hình
    pub async fn save_config_snapshot(
        pool: &SqlitePool,
        config_json: &str,
        description: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO config_snapshots (config_data, description) VALUES (?, ?)",
        )
        .bind(config_json)
        .bind(description)
        .execute(pool)
        .await?;

        info!("Đã lưu config snapshot: {}", description);
        Ok(())
    }

    /// Lưu price data point
    pub async fn save_price_data(
        pool: &SqlitePool,
        asset_type: &str,
        identifier: &str,
        price: f64,
        volume: Option<f64>,
        currency: &str,
        source: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO price_data (asset_type, identifier, price, volume, currency, source)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(asset_type)
        .bind(identifier)
        .bind(price)
        .bind(volume)
        .bind(currency)
        .bind(source)
        .execute(pool)
        .await?;

        Ok(())
    }
}
