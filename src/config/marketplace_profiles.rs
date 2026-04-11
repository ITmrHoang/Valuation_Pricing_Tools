// === Marketplace Profiles - Cấu hình cho từng sàn giao dịch ===
// Mỗi marketplace có profile riêng: endpoints, rate limits, attribute mappings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Profile cấu hình cho một marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceProfile {
    /// Có bật scraping cho marketplace này không
    pub enabled: bool,
    /// URL gốc API
    pub base_url: String,
    /// API key (nếu cần)
    #[serde(default)]
    pub api_key: String,
    /// Thời gian chờ giữa các request (ms)
    pub rate_limit_ms: u64,
    /// Mapping tên field từ marketplace → internal field
    #[serde(default)]
    pub field_mappings: HashMap<String, String>,
    /// Headers bổ sung cho request
    #[serde(default)]
    pub custom_headers: HashMap<String, String>,
}

/// Danh sách các marketplace được hỗ trợ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketplaceType {
    OpenSea,
    MagicEden,
    Blur,
    Rarible,
    /// Marketplace tuỳ chỉnh
    Custom,
}

impl std::fmt::Display for MarketplaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketplaceType::OpenSea => write!(f, "opensea"),
            MarketplaceType::MagicEden => write!(f, "magic_eden"),
            MarketplaceType::Blur => write!(f, "blur"),
            MarketplaceType::Rarible => write!(f, "rarible"),
            MarketplaceType::Custom => write!(f, "custom"),
        }
    }
}

impl MarketplaceProfile {
    /// Tạo profile mặc định cho OpenSea
    pub fn default_opensea() -> Self {
        let mut field_mappings = HashMap::new();
        // Mapping từ OpenSea API field → internal field
        field_mappings.insert("stats.floor_price".to_string(), "floor_price".to_string());
        field_mappings.insert("stats.total_volume".to_string(), "total_volume".to_string());
        field_mappings.insert("stats.num_owners".to_string(), "holder_count".to_string());
        field_mappings.insert("stats.market_cap".to_string(), "market_cap".to_string());
        field_mappings.insert("stats.one_day_volume".to_string(), "volume_24h".to_string());

        let mut custom_headers = HashMap::new();
        custom_headers.insert("Accept".to_string(), "application/json".to_string());

        Self {
            enabled: true,
            base_url: "https://api.opensea.io/api/v2".to_string(),
            api_key: String::new(),
            rate_limit_ms: 500,
            field_mappings,
            custom_headers,
        }
    }

    /// Tạo profile mặc định cho Magic Eden
    pub fn default_magic_eden() -> Self {
        let mut field_mappings = HashMap::new();
        field_mappings.insert("floorPrice".to_string(), "floor_price".to_string());
        field_mappings.insert("volumeAll".to_string(), "total_volume".to_string());
        field_mappings.insert("listedCount".to_string(), "listed_count".to_string());

        Self {
            enabled: true,
            base_url: "https://api-mainnet.magiceden.dev/v2".to_string(),
            api_key: String::new(),
            rate_limit_ms: 500,
            field_mappings,
            custom_headers: HashMap::new(),
        }
    }

    /// Tạo profile mặc định cho Blur
    pub fn default_blur() -> Self {
        let mut field_mappings = HashMap::new();
        field_mappings.insert("floorPrice".to_string(), "floor_price".to_string());
        field_mappings.insert("totalVolume".to_string(), "total_volume".to_string());

        Self {
            enabled: false,
            base_url: "https://api.blur.io/v1".to_string(),
            api_key: String::new(),
            rate_limit_ms: 1000,
            field_mappings,
            custom_headers: HashMap::new(),
        }
    }
}
