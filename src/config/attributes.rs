// === Dynamic Attributes - Thuộc tính định giá động ===
// Định nghĩa các thuộc tính có thể cấu hình runtime cho từng loại tài sản

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Loại dữ liệu của attribute
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AttributeType {
    /// Số thực (giá, volume, %)
    Numeric,
    /// Chuỗi text (tên collection, symbol)
    Text,
    /// Đúng/Sai (verified, blue_chip)
    Boolean,
    /// Danh sách giá trị (traits, categories)
    List,
    /// Tỷ lệ phần trăm (0.0 - 1.0)
    Percentage,
}

/// Định nghĩa một thuộc tính động
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDefinition {
    /// Tên thuộc tính (ví dụ: "rarity_score", "floor_price")
    pub name: String,
    /// Mô tả thuộc tính
    pub description: String,
    /// Loại dữ liệu
    pub attr_type: AttributeType,
    /// Trọng số trong tính toán scoring (0.0 - 1.0)
    pub weight: f64,
    /// Giá trị tối thiểu (cho Numeric)
    pub min_value: Option<f64>,
    /// Giá trị tối đa (cho Numeric)
    pub max_value: Option<f64>,
    /// Giá trị mặc định
    pub default_value: Option<String>,
    /// Có bắt buộc hay không
    pub required: bool,
    /// Thuộc tính có ảnh hưởng tích cực (+) hay tiêu cực (-) đến giá trị
    pub positive_impact: bool,
}

/// Tập hợp các attributes áp dụng cho một loại tài sản
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeSet {
    /// Tên tập thuộc tính (ví dụ: "nft_default", "stock_crypto")
    pub name: String,
    /// Mô tả
    pub description: String,
    /// Danh sách thuộc tính
    pub attributes: Vec<AttributeDefinition>,
    /// Metadata bổ sung
    pub metadata: HashMap<String, String>,
}

impl AttributeSet {
    /// Tạo tập thuộc tính mặc định cho NFT
    pub fn default_nft_attributes() -> Self {
        Self {
            name: "nft_default".to_string(),
            description: "Tập thuộc tính mặc định cho định giá NFT".to_string(),
            attributes: vec![
                AttributeDefinition {
                    name: "rarity_score".to_string(),
                    description: "Điểm hiếm có của NFT trong collection".to_string(),
                    attr_type: AttributeType::Numeric,
                    weight: 0.25,
                    min_value: Some(0.0),
                    max_value: Some(100.0),
                    default_value: Some("50.0".to_string()),
                    required: true,
                    positive_impact: true,
                },
                AttributeDefinition {
                    name: "floor_price".to_string(),
                    description: "Giá sàn hiện tại của collection (ETH/SOL)".to_string(),
                    attr_type: AttributeType::Numeric,
                    weight: 0.20,
                    min_value: Some(0.0),
                    max_value: None,
                    default_value: None,
                    required: true,
                    positive_impact: true,
                },
                AttributeDefinition {
                    name: "volume_24h".to_string(),
                    description: "Khối lượng giao dịch 24 giờ".to_string(),
                    attr_type: AttributeType::Numeric,
                    weight: 0.15,
                    min_value: Some(0.0),
                    max_value: None,
                    default_value: Some("0.0".to_string()),
                    required: false,
                    positive_impact: true,
                },
                AttributeDefinition {
                    name: "market_cap".to_string(),
                    description: "Vốn hoá thị trường của collection".to_string(),
                    attr_type: AttributeType::Numeric,
                    weight: 0.15,
                    min_value: Some(0.0),
                    max_value: None,
                    default_value: None,
                    required: false,
                    positive_impact: true,
                },
                AttributeDefinition {
                    name: "trend_direction".to_string(),
                    description: "Xu hướng giá: tăng(+1), giảm(-1), sideway(0)".to_string(),
                    attr_type: AttributeType::Numeric,
                    weight: 0.10,
                    min_value: Some(-1.0),
                    max_value: Some(1.0),
                    default_value: Some("0.0".to_string()),
                    required: false,
                    positive_impact: true,
                },
                AttributeDefinition {
                    name: "liquidity_score".to_string(),
                    description: "Điểm thanh khoản (dựa trên số lượng giao dịch)".to_string(),
                    attr_type: AttributeType::Percentage,
                    weight: 0.10,
                    min_value: Some(0.0),
                    max_value: Some(1.0),
                    default_value: Some("0.5".to_string()),
                    required: false,
                    positive_impact: true,
                },
                AttributeDefinition {
                    name: "holder_count".to_string(),
                    description: "Số lượng holder duy nhất".to_string(),
                    attr_type: AttributeType::Numeric,
                    weight: 0.05,
                    min_value: Some(0.0),
                    max_value: None,
                    default_value: None,
                    required: false,
                    positive_impact: true,
                },
            ],
            metadata: HashMap::new(),
        }
    }

    /// Tạo tập thuộc tính mặc định cho Stock
    pub fn default_stock_attributes() -> Self {
        Self {
            name: "stock_default".to_string(),
            description: "Tập thuộc tính mặc định cho phân tích cổ phiếu".to_string(),
            attributes: vec![
                AttributeDefinition {
                    name: "volatility".to_string(),
                    description: "Biến động giá (standard deviation)".to_string(),
                    attr_type: AttributeType::Numeric,
                    weight: 0.20,
                    min_value: Some(0.0),
                    max_value: None,
                    default_value: None,
                    required: true,
                    positive_impact: false,
                },
                AttributeDefinition {
                    name: "price_momentum".to_string(),
                    description: "Động lượng giá (% thay đổi)".to_string(),
                    attr_type: AttributeType::Percentage,
                    weight: 0.15,
                    min_value: Some(-1.0),
                    max_value: Some(1.0),
                    default_value: Some("0.0".to_string()),
                    required: true,
                    positive_impact: true,
                },
                AttributeDefinition {
                    name: "volume_change".to_string(),
                    description: "Thay đổi khối lượng giao dịch (%)".to_string(),
                    attr_type: AttributeType::Percentage,
                    weight: 0.15,
                    min_value: Some(-1.0),
                    max_value: Some(10.0),
                    default_value: Some("0.0".to_string()),
                    required: false,
                    positive_impact: true,
                },
                AttributeDefinition {
                    name: "rsi".to_string(),
                    description: "Relative Strength Index (0-100)".to_string(),
                    attr_type: AttributeType::Numeric,
                    weight: 0.15,
                    min_value: Some(0.0),
                    max_value: Some(100.0),
                    default_value: Some("50.0".to_string()),
                    required: false,
                    positive_impact: true, // Xử lý phức tạp hơn trong engine
                },
                AttributeDefinition {
                    name: "moving_avg_signal".to_string(),
                    description: "Tín hiệu trung bình động: mua(+1), bán(-1), trung lập(0)".to_string(),
                    attr_type: AttributeType::Numeric,
                    weight: 0.15,
                    min_value: Some(-1.0),
                    max_value: Some(1.0),
                    default_value: Some("0.0".to_string()),
                    required: false,
                    positive_impact: true,
                },
                AttributeDefinition {
                    name: "bollinger_position".to_string(),
                    description: "Vị trí trong Bollinger Bands (0=lower, 1=upper)".to_string(),
                    attr_type: AttributeType::Percentage,
                    weight: 0.10,
                    min_value: Some(0.0),
                    max_value: Some(1.0),
                    default_value: Some("0.5".to_string()),
                    required: false,
                    positive_impact: true,
                },
                AttributeDefinition {
                    name: "trend_strength".to_string(),
                    description: "Sức mạnh xu hướng (0-100)".to_string(),
                    attr_type: AttributeType::Numeric,
                    weight: 0.10,
                    min_value: Some(0.0),
                    max_value: Some(100.0),
                    default_value: Some("50.0".to_string()),
                    required: false,
                    positive_impact: true,
                },
            ],
            metadata: HashMap::new(),
        }
    }

    /// Tìm attribute theo tên
    pub fn find_attribute(&self, name: &str) -> Option<&AttributeDefinition> {
        self.attributes.iter().find(|a| a.name == name)
    }

    /// Cập nhật trọng số cho một attribute
    pub fn update_weight(&mut self, name: &str, new_weight: f64) -> bool {
        if let Some(attr) = self.attributes.iter_mut().find(|a| a.name == name) {
            attr.weight = new_weight.clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }

    /// Chuẩn hoá trọng số để tổng = 1.0
    pub fn normalize_weights(&mut self) {
        let total: f64 = self.attributes.iter().map(|a| a.weight).sum();
        if total > 0.0 {
            for attr in &mut self.attributes {
                attr.weight /= total;
            }
        }
    }
}

/// Cấu hình trọng số cho scoring engine
/// Key: tên attribute, Value: trọng số (0.0 - 1.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightConfig {
    #[serde(flatten)]
    pub weights: HashMap<String, f64>,
}

impl WeightConfig {
    /// Trọng số mặc định cho NFT
    pub fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert("rarity".to_string(), 0.25);
        weights.insert("floor_price".to_string(), 0.20);
        weights.insert("volume_24h".to_string(), 0.15);
        weights.insert("market_cap".to_string(), 0.15);
        weights.insert("trend".to_string(), 0.10);
        weights.insert("liquidity".to_string(), 0.10);
        weights.insert("holder_count".to_string(), 0.05);
        Self { weights }
    }

    /// Trọng số mặc định cho Stock
    pub fn default_stock() -> Self {
        let mut weights = HashMap::new();
        weights.insert("volatility".to_string(), 0.20);
        weights.insert("price_momentum".to_string(), 0.15);
        weights.insert("volume_change".to_string(), 0.15);
        weights.insert("rsi".to_string(), 0.15);
        weights.insert("moving_avg_signal".to_string(), 0.15);
        weights.insert("bollinger_position".to_string(), 0.10);
        weights.insert("trend_strength".to_string(), 0.10);
        Self { weights }
    }

    /// Trọng số mặc định cho Fundamental Analysis
    pub fn default_fundamental() -> Self {
        let mut weights = HashMap::new();
        weights.insert("pe_ratio".to_string(), 0.20);
        weights.insert("pb_ratio".to_string(), 0.15);
        weights.insert("eps_growth".to_string(), 0.20);
        weights.insert("dividend".to_string(), 0.15);
        weights.insert("dcf_margin".to_string(), 0.15);
        weights.insert("stability".to_string(), 0.15);
        Self { weights }
    }

    /// Lấy trọng số theo tên attribute
    pub fn get_weight(&self, name: &str) -> f64 {
        *self.weights.get(name).unwrap_or(&0.0)
    }

    /// Cập nhật trọng số
    pub fn set_weight(&mut self, name: &str, value: f64) {
        self.weights.insert(name.to_string(), value.clamp(0.0, 1.0));
    }

    /// Chuẩn hoá tổng trọng số = 1.0
    pub fn normalize(&mut self) {
        let total: f64 = self.weights.values().sum();
        if total > 0.0 {
            for value in self.weights.values_mut() {
                *value /= total;
            }
        }
    }
}
