// === Mô-đun cấu hình động cho hệ thống định giá ===
// Quản lý dynamic attributes, hot-reload config, marketplace profiles

pub mod attributes;
pub mod marketplace_profiles;

use std::sync::Arc;
use arc_swap::ArcSwap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use self::attributes::{AttributeSet, WeightConfig};
use self::marketplace_profiles::MarketplaceProfile;
use crate::engine::fundamental_analysis::FundamentalConfig;
use crate::scrapers::proxy_pool::ProxyPoolConfig;



/// Cấu hình server
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Cấu hình database
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

/// Cấu hình logging
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

/// Cấu hình scraping chung
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScrapingConfig {
    /// Thời gian chờ giữa các request (ms)
    pub rate_limit_ms: u64,
    /// Số lần retry tối đa
    pub max_retries: u32,
    /// Timeout cho mỗi request (giây)
    pub request_timeout_secs: u64,
    /// User-Agent header
    pub user_agent: String,
}

/// Cấu hình stock data provider
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StockDataConfig {
    pub provider: String,
    pub base_url: String,
    /// Số ngày lịch sử cần lấy
    pub history_days: u32,
    /// Interval: 1d, 1wk, 1mo
    pub interval: String,
}

/// Cấu hình scoring
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoringConfig {
    /// Trọng số cho các thuộc tính NFT
    pub nft_weights: WeightConfig,
    /// Trọng số cho các thuộc tính Stock (technical)
    pub stock_weights: WeightConfig,
    /// Trọng số cho các chỉ số fundamental
    #[serde(default = "WeightConfig::default_fundamental")]
    pub fundamental_weights: WeightConfig,
}

/// Cấu hình CrewAI integration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrewAiConfig {
    pub enabled: bool,
    pub webhook_path: String,
    pub webhook_secret: String,
}

/// Cấu hình tổng hợp của toàn bộ ứng dụng
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub scraping: ScrapingConfig,
    pub marketplaces: std::collections::HashMap<String, MarketplaceProfile>,
    pub stock_data: StockDataConfig,
    pub scoring: ScoringConfig,
    pub crew_ai: CrewAiConfig,
    /// Cấu hình proxy pool cho scraping
    #[serde(default)]
    pub proxy: ProxyPoolConfig,
    /// Cấu hình fundamental analysis
    #[serde(default)]
    pub fundamental: FundamentalConfig,
}

/// Dynamic Config Manager - quản lý cấu hình với khả năng hot-reload
/// Sử dụng ArcSwap để cho phép đọc lock-free và cập nhật atomic
pub struct DynamicConfigManager {
    /// Config hiện tại, có thể swap atomically
    config: ArcSwap<AppConfig>,
    /// Tập hợp dynamic attributes đang active
    active_attributes: ArcSwap<AttributeSet>,
}

impl DynamicConfigManager {
    /// Khởi tạo ConfigManager từ file cấu hình
    pub fn new(config: AppConfig) -> Self {
        let default_attributes = AttributeSet::default_nft_attributes();
        info!("Khởi tạo DynamicConfigManager thành công");
        
        Self {
            config: ArcSwap::from_pointee(config),
            active_attributes: ArcSwap::from_pointee(default_attributes),
        }
    }

    /// Lấy config hiện tại (lock-free read)
    pub fn get_config(&self) -> Arc<AppConfig> {
        self.config.load_full()
    }

    /// Cập nhật config mới (atomic swap)
    pub fn update_config(&self, new_config: AppConfig) {
        info!("Cập nhật cấu hình mới");
        self.config.store(Arc::new(new_config));
    }

    /// Lấy active attributes
    pub fn get_attributes(&self) -> Arc<AttributeSet> {
        self.active_attributes.load_full()
    }

    /// Cập nhật dynamic attributes
    pub fn update_attributes(&self, new_attrs: AttributeSet) {
        info!("Cập nhật dynamic attributes: {} thuộc tính", new_attrs.attributes.len());
        self.active_attributes.store(Arc::new(new_attrs));
    }

    /// Cập nhật trọng số scoring cho NFT
    pub fn update_nft_weights(&self, weights: WeightConfig) {
        let mut config = (*self.config.load_full()).clone();
        config.scoring.nft_weights = weights;
        self.config.store(Arc::new(config));
        info!("Cập nhật trọng số NFT scoring");
    }

    /// Cập nhật trọng số scoring cho Stock
    pub fn update_stock_weights(&self, weights: WeightConfig) {
        let mut config = (*self.config.load_full()).clone();
        config.scoring.stock_weights = weights;
        self.config.store(Arc::new(config));
        info!("Cập nhật trọng số Stock scoring");
    }
}

/// Load cấu hình từ file TOML
pub fn load_config(config_path: &str) -> Result<AppConfig> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name(config_path))
        .add_source(config::Environment::with_prefix("APP").separator("_"))
        .build()?;

    let app_config: AppConfig = settings.try_deserialize()?;
    info!("Đã load cấu hình từ: {}", config_path);
    Ok(app_config)
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            database: DatabaseConfig {
                url: "sqlite://data/pricing.db".to_string(),
                max_connections: 5,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            scraping: ScrapingConfig {
                rate_limit_ms: 1000,
                max_retries: 3,
                request_timeout_secs: 30,
                user_agent: "ValuationPricingTools/0.1.0".to_string(),
            },
            marketplaces: std::collections::HashMap::new(),
            stock_data: StockDataConfig {
                provider: "yahoo_finance".to_string(),
                base_url: "https://query1.finance.yahoo.com/v8/finance".to_string(),
                history_days: 365,
                interval: "1d".to_string(),
            },
            scoring: ScoringConfig {
                nft_weights: WeightConfig::default(),
                stock_weights: WeightConfig::default_stock(),
                fundamental_weights: WeightConfig::default_fundamental(),
            },
            crew_ai: CrewAiConfig {
                enabled: true,
                webhook_path: "/api/v1/crew/webhook".to_string(),
                webhook_secret: String::new(),
            },
            proxy: ProxyPoolConfig::default(),
            fundamental: FundamentalConfig::default(),
        }
    }
}
