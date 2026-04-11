// === Scraper Module - Thu thập dữ liệu từ marketplaces ===
// Async scraping với rate limiting, retry logic, data validation

pub mod opensea;
pub mod magic_eden;
pub mod stock_data;
pub mod proxy_pool;

use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// Trait chung cho tất cả marketplace scrapers
#[async_trait]
pub trait MarketplaceScraper: Send + Sync {
    /// Tên marketplace
    fn name(&self) -> &str;

    /// Lấy thông tin collection
    async fn fetch_collection(&self, collection_slug: &str) -> Result<CollectionInfo>;

    /// Lấy thông tin một NFT cụ thể
    async fn fetch_nft(&self, contract: &str, token_id: &str) -> Result<NftInfo>;

    /// Kiểm tra kết nối API
    async fn health_check(&self) -> Result<bool>;
}

/// Thông tin collection từ marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    /// Tên collection
    pub name: String,
    /// Slug/ID
    pub slug: String,
    /// Marketplace nguồn
    pub source: String,
    /// Giá sàn
    pub floor_price: f64,
    /// Đơn vị tiền tệ
    pub currency: String,
    /// Khối lượng 24h
    pub volume_24h: f64,
    /// Tổng khối lượng
    pub total_volume: f64,
    /// Vốn hoá
    pub market_cap: f64,
    /// Số holder
    pub holder_count: u64,
    /// Số item đang list
    pub listed_count: u64,
    /// Tổng supply
    pub total_supply: u64,
    /// Giá sàn 7 ngày trước
    pub floor_price_7d_ago: Option<f64>,
    /// Dữ liệu thô từ API
    pub raw_data: Option<serde_json::Value>,
}

/// Thông tin một NFT item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftInfo {
    /// Contract address
    pub contract: String,
    /// Token ID
    pub token_id: String,
    /// Tên
    pub name: String,
    /// Rarity score
    pub rarity_score: f64,
    /// Giá bán cuối cùng
    pub last_sale_price: Option<f64>,
    /// Traits
    pub traits: std::collections::HashMap<String, String>,
    /// Image URL
    pub image_url: Option<String>,
}

/// HTTP client wrapper với rate limiting và proxy support
pub struct RateLimitedClient {
    /// Client mặc định (không proxy)
    client: reqwest::Client,
    /// Thời gian chờ giữa các request
    rate_limit: Duration,
    /// User-Agent
    user_agent: String,
    /// Proxy pool (tuỳ chọn)
    proxy_pool: Option<Arc<proxy_pool::ProxyPool>>,
    /// Timeout cho request (giây)
    timeout_secs: u64,
}

impl RateLimitedClient {
    /// Tạo client mới không proxy
    pub fn new(rate_limit_ms: u64, user_agent: &str, timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(user_agent)
            .build()
            .expect("Không thể tạo HTTP client");

        Self {
            client,
            rate_limit: Duration::from_millis(rate_limit_ms),
            user_agent: user_agent.to_string(),
            proxy_pool: None,
            timeout_secs,
        }
    }

    /// Tạo client với proxy pool
    pub fn new_with_proxy(
        rate_limit_ms: u64,
        user_agent: &str,
        timeout_secs: u64,
        proxy_pool: Arc<proxy_pool::ProxyPool>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(user_agent)
            .build()
            .expect("Không thể tạo HTTP client");

        info!("Tạo RateLimitedClient với proxy pool ({} proxies)", proxy_pool.alive_count());

        Self {
            client,
            rate_limit: Duration::from_millis(rate_limit_ms),
            user_agent: user_agent.to_string(),
            proxy_pool: Some(proxy_pool),
            timeout_secs,
        }
    }

    /// Lấy HTTP client phù hợp (có proxy hoặc không)
    fn get_client_for_request(&self) -> (reqwest::Client, Option<String>) {
        // Nếu có proxy pool và còn proxy khả dụng
        if let Some(pool) = &self.proxy_pool {
            if let Some(proxy_config) = pool.get_next_proxy() {
                let proxy_url = proxy_config.to_url();
                match proxy_pool::ProxyPool::create_proxied_client(
                    &proxy_config, self.timeout_secs, &self.user_agent,
                ) {
                    Ok(proxied_client) => {
                        return (proxied_client, Some(proxy_url));
                    }
                    Err(e) => {
                        warn!("Không tạo được proxied client: {}. Fallback direct.", e);
                    }
                }
            }
        }
        // Fallback: dùng client trực tiếp
        (self.client.clone(), None)
    }

    /// Gửi GET request với retry logic và proxy rotation
    pub async fn get_with_retry(
        &self,
        url: &str,
        headers: &[(String, String)],
        max_retries: u32,
    ) -> Result<serde_json::Value> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                // Exponential backoff
                let delay = self.rate_limit * 2u32.pow(attempt - 1);
                info!("Retry #{} sau {:?}", attempt, delay);
                tokio::time::sleep(delay).await;
            }

            // Lấy client (có thể qua proxy)
            let (client, proxy_url) = self.get_client_for_request();

            if let Some(ref pu) = proxy_url {
                info!("Request qua proxy: {} -> {}", pu, url);
            }

            let mut request = client.get(url);

            // Thêm custom headers
            for (key, value) in headers {
                request = request.header(key.as_str(), value.as_str());
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        // Báo thành công cho proxy pool
                        if let (Some(pool), Some(ref pu)) = (&self.proxy_pool, &proxy_url) {
                            pool.report_success(pu);
                        }
                        let json: serde_json::Value = response.json().await?;
                        // Rate limiting
                        tokio::time::sleep(self.rate_limit).await;
                        return Ok(json);
                    } else if response.status().as_u16() == 429 {
                        // Rate limited - báo lỗi proxy và chờ lâu hơn
                        warn!("Rate limited bởi {}. Đợi...", url);
                        if let (Some(pool), Some(ref pu)) = (&self.proxy_pool, &proxy_url) {
                            pool.report_failure(pu);
                        }
                        tokio::time::sleep(self.rate_limit * 5).await;
                        last_error = Some(anyhow::anyhow!("Rate limited: {}", response.status()));
                    } else {
                        if let (Some(pool), Some(ref pu)) = (&self.proxy_pool, &proxy_url) {
                            pool.report_failure(pu);
                        }
                        last_error = Some(anyhow::anyhow!(
                            "HTTP error {}: {}",
                            response.status(),
                            url
                        ));
                    }
                }
                Err(e) => {
                    warn!("Request failed: {} - {}", url, e);
                    if let (Some(pool), Some(ref pu)) = (&self.proxy_pool, &proxy_url) {
                        pool.report_failure(pu);
                    }
                    last_error = Some(anyhow::anyhow!("Request failed: {}", e));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Max retries exceeded cho {}", url)))
    }
}
