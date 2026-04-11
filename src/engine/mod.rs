// === Pricing Engine - Lõi hệ thống định giá ===
// Orchestrator điều phối NFT và Stock valuators

pub mod nft_valuator;
pub mod stock_valuator;
pub mod scoring;
pub mod fundamental_analysis;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Loại tài sản cần định giá
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    /// NFT trên các marketplace Web3
    Nft,
    /// Cổ phiếu truyền thống
    Stock,
    /// Token crypto
    Crypto,
}

/// Mức độ tin cậy của kết quả định giá
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    /// Rất cao (>90%) - đủ dữ liệu, market stable
    VeryHigh,
    /// Cao (70-90%) - đủ dữ liệu, market biến động vừa
    High,
    /// Trung bình (50-70%) - thiếu một số dữ liệu
    Medium,
    /// Thấp (30-50%) - dữ liệu hạn chế
    Low,
    /// Rất thấp (<30%) - dữ liệu không đáng tin
    VeryLow,
}

/// Xu hướng giá
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    /// Xu hướng tăng mạnh
    StrongBullish,
    /// Xu hướng tăng
    Bullish,
    /// Đi ngang
    Neutral,
    /// Xu hướng giảm
    Bearish,
    /// Xu hướng giảm mạnh
    StrongBearish,
}

/// Khuyến nghị mua/bán cho tài sản
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Recommendation {
    /// Khuyến nghị mua mạnh - giá rất hấp dẫn
    StrongBuy,
    /// Khuyến nghị mua
    Buy,
    /// Giữ nguyên vị thế
    Hold,
    /// Khuyến nghị bán
    Sell,
    /// Khuyến nghị bán mạnh - giá quá cao
    StrongSell,
}

/// Kết quả định giá chuẩn hoá
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationResult {
    /// ID duy nhất cho kết quả
    pub id: String,
    /// Loại tài sản
    pub asset_type: AssetType,
    /// Tên/Symbol tài sản
    pub asset_identifier: String,
    /// Marketplace nguồn
    pub source: String,
    /// Giá ước tính
    pub estimated_price: f64,
    /// Đơn vị tiền tệ (ETH, SOL, USD, ...)
    pub currency: String,
    /// Điểm tổng hợp (0-100)
    pub composite_score: f64,
    /// Xu hướng
    pub trend: TrendDirection,
    /// Mức độ tin cậy
    pub confidence: ConfidenceLevel,
    /// Phần trăm tin cậy (0.0 - 1.0)
    pub confidence_pct: f64,
    /// Điểm chi tiết theo từng thuộc tính
    pub attribute_scores: HashMap<String, f64>,
    /// Dữ liệu thô từ marketplace
    pub raw_data: Option<serde_json::Value>,
    /// Thời điểm định giá
    pub timestamp: DateTime<Utc>,
    /// Ghi chú bổ sung
    pub notes: Vec<String>,
    /// Khuyến nghị mua/bán (nếu có phân tích fundamental)
    pub recommendation: Option<Recommendation>,
}

/// Trait chung cho tất cả Valuation Engine
#[async_trait]
pub trait PricingEngine: Send + Sync {
    /// Loại tài sản mà engine xử lý
    fn asset_type(&self) -> AssetType;

    /// Định giá một tài sản cụ thể
    async fn valuate(&self, request: &ValuationRequest) -> anyhow::Result<ValuationResult>;

    /// Định giá hàng loạt (batch) - tối ưu song song
    async fn valuate_batch(&self, requests: &[ValuationRequest]) -> Vec<anyhow::Result<ValuationResult>>;
}

/// Yêu cầu định giá
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationRequest {
    /// Loại tài sản
    pub asset_type: AssetType,
    /// Tên/Symbol/Address tài sản
    pub identifier: String,
    /// Marketplace cụ thể (tuỳ chọn)
    pub marketplace: Option<String>,
    /// Dữ liệu bổ sung từ CrewAI hoặc nguồn khác
    pub additional_data: Option<serde_json::Value>,
    /// Override trọng số
    pub weight_overrides: Option<HashMap<String, f64>>,
}

impl ValuationResult {
    /// Tạo kết quả mới với giá trị mặc định
    pub fn new(asset_type: AssetType, identifier: String, source: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            asset_type,
            asset_identifier: identifier,
            source,
            estimated_price: 0.0,
            currency: "USD".to_string(),
            composite_score: 0.0,
            trend: TrendDirection::Neutral,
            confidence: ConfidenceLevel::Low,
            confidence_pct: 0.0,
            attribute_scores: HashMap::new(),
            raw_data: None,
            timestamp: Utc::now(),
            notes: Vec::new(),
            recommendation: None,
        }
    }

    /// Xác định mức độ tin cậy từ phần trăm
    pub fn calculate_confidence(&mut self) {
        self.confidence = match self.confidence_pct {
            p if p >= 0.9 => ConfidenceLevel::VeryHigh,
            p if p >= 0.7 => ConfidenceLevel::High,
            p if p >= 0.5 => ConfidenceLevel::Medium,
            p if p >= 0.3 => ConfidenceLevel::Low,
            _ => ConfidenceLevel::VeryLow,
        };
    }
}
