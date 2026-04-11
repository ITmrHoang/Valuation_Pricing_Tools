// === API Models - Request/Response structures cho REST API ===

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request định giá NFT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuateNftRequest {
    /// Tên collection hoặc contract address
    pub identifier: String,
    /// Marketplace cụ thể (tuỳ chọn)
    pub marketplace: Option<String>,
    /// Dữ liệu collection (nếu đã có sẵn)
    pub data: Option<serde_json::Value>,
    /// Override trọng số
    pub weight_overrides: Option<HashMap<String, f64>>,
}

/// Request phân tích cổ phiếu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuateStockRequest {
    /// Mã cổ phiếu (ticker symbol)
    pub symbol: String,
    /// Dữ liệu OHLCV (nếu đã có sẵn)
    pub data: Option<serde_json::Value>,
    /// Override trọng số
    pub weight_overrides: Option<HashMap<String, f64>>,
}

/// Request batch valuation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchValuationRequest {
    pub requests: Vec<BatchItemRequest>,
}

/// Một item trong batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemRequest {
    /// Loại: "nft" hoặc "stock"
    pub asset_type: String,
    /// Identifier
    pub identifier: String,
    /// Marketplace (cho NFT)
    pub marketplace: Option<String>,
    /// Dữ liệu bổ sung
    pub data: Option<serde_json::Value>,
}

/// Request phân tích fundamental cổ phiếu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuateFundamentalRequest {
    /// Mã cổ phiếu
    pub symbol: String,
    /// Dữ liệu tài chính fundamental
    pub fundamental_data: serde_json::Value,
    /// Dữ liệu OHLCV (tuỳ chọn, để kết hợp technical analysis)
    pub bars: Option<serde_json::Value>,
    /// Override trọng số
    pub weight_overrides: Option<HashMap<String, f64>>,
}

/// Response chuẩn
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub status: String,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Tạo response thành công
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            data: Some(data),
            message: None,
        }
    }

    /// Tạo response lỗi
    pub fn error(message: &str) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            message: Some(message.to_string()),
        }
    }
}
