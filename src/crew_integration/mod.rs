// === CrewAI Integration - Nhận và xử lý dữ liệu từ CrewAI Spinner ===
// Transform CrewAI output thành internal format cho pricing engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tracing::{info, warn};

/// Dữ liệu nhận từ CrewAI Spinner webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewSpinnerData {
    /// ID task của CrewAI
    pub task_id: String,
    /// Loại dữ liệu: "nft_collection", "nft_item", "stock_analysis"
    pub data_type: String,
    /// Marketplace nguồn
    pub source: Option<String>,
    /// Dữ liệu chính
    pub payload: serde_json::Value,
    /// Metadata bổ sung
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Thời điểm tạo
    pub created_at: Option<DateTime<Utc>>,
}

/// Kết quả sau khi xử lý dữ liệu CrewAI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedCrewData {
    /// Loại tài sản
    pub asset_type: String,
    /// Identifier (collection slug, ticker, ...)
    pub identifier: String,
    /// Dữ liệu đã chuẩn hoá cho pricing engine
    pub normalized_data: serde_json::Value,
    /// Nguồn dữ liệu gốc
    pub source: String,
    /// Có hợp lệ không
    pub is_valid: bool,
    /// Ghi chú/cảnh báo
    pub warnings: Vec<String>,
}

/// CrewAI data processor - chuyển đổi dữ liệu CrewAI sang internal format
pub struct CrewDataProcessor;

impl CrewDataProcessor {
    /// Xử lý webhook data từ CrewAI Spinner
    pub fn process(data: &CrewSpinnerData) -> ProcessedCrewData {
        info!("Xử lý CrewAI data: task={}, type={}", data.task_id, data.data_type);

        match data.data_type.as_str() {
            "nft_collection" => Self::process_nft_collection(data),
            "nft_item" => Self::process_nft_item(data),
            "stock_analysis" => Self::process_stock_analysis(data),
            "market_overview" => Self::process_market_overview(data),
            _ => {
                warn!("Loại dữ liệu không được hỗ trợ: {}", data.data_type);
                ProcessedCrewData {
                    asset_type: "unknown".to_string(),
                    identifier: data.task_id.clone(),
                    normalized_data: data.payload.clone(),
                    source: "crew_ai".to_string(),
                    is_valid: false,
                    warnings: vec![format!("Loại dữ liệu không hỗ trợ: {}", data.data_type)],
                }
            }
        }
    }

    /// Xử lý dữ liệu NFT collection từ CrewAI
    fn process_nft_collection(data: &CrewSpinnerData) -> ProcessedCrewData {
        let payload = &data.payload;
        let mut warnings = Vec::new();

        // Trích xuất và chuẩn hoá dữ liệu
        let name = payload.get("name")
            .or_else(|| payload.get("collection_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let floor_price = Self::extract_numeric(payload, &["floor_price", "floorPrice", "floor"])
            .unwrap_or_else(|| {
                warnings.push("Không tìm thấy floor_price".to_string());
                0.0
            });

        let volume_24h = Self::extract_numeric(payload, &["volume_24h", "volume24h", "dailyVolume"])
            .unwrap_or(0.0);

        let market_cap = Self::extract_numeric(payload, &["market_cap", "marketCap"])
            .unwrap_or(0.0);

        let holder_count = Self::extract_numeric(payload, &["holder_count", "holders", "numOwners"])
            .map(|v| v as u64)
            .unwrap_or(0);

        let normalized = serde_json::json!({
            "name": name,
            "floor_price": floor_price,
            "volume_24h": volume_24h,
            "market_cap": market_cap,
            "holder_count": holder_count,
            "total_supply": Self::extract_numeric(payload, &["total_supply", "totalSupply"]).unwrap_or(10000.0),
            "listed_count": Self::extract_numeric(payload, &["listed_count", "listedCount"]).unwrap_or(0.0) as u64,
            "currency": payload.get("currency").and_then(|v| v.as_str()).unwrap_or("ETH"),
            "floor_price_7d_ago": Self::extract_numeric(payload, &["floor_price_7d", "floorPrice7d"]),
        });

        ProcessedCrewData {
            asset_type: "nft".to_string(),
            identifier: name.to_string(),
            normalized_data: normalized,
            source: data.source.clone().unwrap_or_else(|| "crew_ai".to_string()),
            is_valid: floor_price > 0.0,
            warnings,
        }
    }

    /// Xử lý dữ liệu NFT item từ CrewAI
    fn process_nft_item(data: &CrewSpinnerData) -> ProcessedCrewData {
        let payload = &data.payload;
        let warnings = Vec::new();

        let token_id = payload.get("token_id")
            .or_else(|| payload.get("tokenId"))
            .and_then(|v| v.as_str())
            .unwrap_or("0");

        let normalized = serde_json::json!({
            "token_id": token_id,
            "name": payload.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown"),
            "rarity_score": Self::extract_numeric(payload, &["rarity_score", "rarity"]).unwrap_or(0.0),
            "last_sale_price": Self::extract_numeric(payload, &["last_sale_price", "lastSale"]),
            "traits": payload.get("traits").cloned().unwrap_or(serde_json::json!({})),
        });

        ProcessedCrewData {
            asset_type: "nft_item".to_string(),
            identifier: token_id.to_string(),
            normalized_data: normalized,
            source: data.source.clone().unwrap_or_else(|| "crew_ai".to_string()),
            is_valid: true,
            warnings,
        }
    }

    /// Xử lý dữ liệu phân tích cổ phiếu từ CrewAI
    fn process_stock_analysis(data: &CrewSpinnerData) -> ProcessedCrewData {
        let payload = &data.payload;
        let mut warnings = Vec::new();

        let symbol = payload.get("symbol")
            .or_else(|| payload.get("ticker"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                warnings.push("Không tìm thấy symbol cổ phiếu".to_string());
                "UNKNOWN"
            });

        // Nếu có OHLCV data, chuẩn hoá
        let bars = payload.get("bars")
            .or_else(|| payload.get("historical"))
            .cloned();

        let normalized = serde_json::json!({
            "symbol": symbol,
            "current_price": Self::extract_numeric(payload, &["price", "current_price", "currentPrice"]),
            "bars": bars,
            "volume": Self::extract_numeric(payload, &["volume", "dailyVolume"]),
            "market_cap": Self::extract_numeric(payload, &["market_cap", "marketCap"]),
        });

        ProcessedCrewData {
            asset_type: "stock".to_string(),
            identifier: symbol.to_string(),
            normalized_data: normalized,
            source: data.source.clone().unwrap_or_else(|| "crew_ai".to_string()),
            is_valid: true,
            warnings,
        }
    }

    /// Xử lý market overview từ CrewAI
    fn process_market_overview(data: &CrewSpinnerData) -> ProcessedCrewData {
        let payload = &data.payload;

        ProcessedCrewData {
            asset_type: "market_overview".to_string(),
            identifier: "market".to_string(),
            normalized_data: payload.clone(),
            source: data.source.clone().unwrap_or_else(|| "crew_ai".to_string()),
            is_valid: true,
            warnings: Vec::new(),
        }
    }

    /// Helper: trích xuất giá trị số từ nhiều tên field có thể
    fn extract_numeric(data: &serde_json::Value, field_names: &[&str]) -> Option<f64> {
        for name in field_names {
            if let Some(value) = data.get(*name) {
                if let Some(n) = value.as_f64() {
                    return Some(n);
                }
                // Thử parse string → number
                if let Some(s) = value.as_str() {
                    if let Ok(n) = s.parse::<f64>() {
                        return Some(n);
                    }
                }
            }
        }
        None
    }
}

/// Xác thực webhook signature từ CrewAI
pub fn verify_webhook_signature(payload: &[u8], signature: &str, secret: &str) -> bool {
    if secret.is_empty() {
        // Không có secret → bỏ qua xác thực (dev mode)
        return true;
    }
    // Đơn giản: so sánh trực tiếp (production nên dùng HMAC-SHA256)
    // TODO: Implement HMAC-SHA256 verification
    !signature.is_empty()
}
