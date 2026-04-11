// === Magic Eden Scraper - Tích hợp Magic Eden API ===
// Hỗ trợ Solana + multi-chain NFT data

use async_trait::async_trait;
use anyhow::Result;
use tracing::{info, warn};

use super::{CollectionInfo, MarketplaceScraper, NftInfo, RateLimitedClient};

/// Magic Eden API scraper
pub struct MagicEdenScraper {
    client: RateLimitedClient,
    base_url: String,
    api_key: String,
}

impl MagicEdenScraper {
    /// Khởi tạo Magic Eden scraper
    pub fn new(base_url: &str, api_key: &str, rate_limit_ms: u64) -> Self {
        let client = RateLimitedClient::new(rate_limit_ms, "ValuationPricingTools/0.1.0", 30);
        info!("Khởi tạo Magic Eden scraper: {}", base_url);

        Self {
            client,
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Headers cho Magic Eden API
    fn api_headers(&self) -> Vec<(String, String)> {
        let mut headers = vec![
            ("Accept".to_string(), "application/json".to_string()),
        ];
        if !self.api_key.is_empty() {
            headers.push(("Authorization".to_string(), format!("Bearer {}", self.api_key)));
        }
        headers
    }
}

#[async_trait]
impl MarketplaceScraper for MagicEdenScraper {
    fn name(&self) -> &str {
        "magic_eden"
    }

    async fn fetch_collection(&self, collection_slug: &str) -> Result<CollectionInfo> {
        info!("Lấy thông tin collection Magic Eden: {}", collection_slug);

        // Lấy stats collection
        let stats_url = format!("{}/collections/{}/stats", self.base_url, collection_slug);
        let stats = self.client.get_with_retry(&stats_url, &self.api_headers(), 3).await?;

        // Lấy thông tin cơ bản
        let info_url = format!("{}/collections/{}", self.base_url, collection_slug);
        let info_data = self.client.get_with_retry(&info_url, &self.api_headers(), 3)
            .await
            .unwrap_or_default();

        // Floor price trên Magic Eden (Solana) tính bằng lamports → SOL
        let floor_price_raw = stats.get("floorPrice")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        // Chuyển lamports → SOL (1 SOL = 1e9 lamports)
        let floor_price = floor_price_raw / 1e9;

        Ok(CollectionInfo {
            name: info_data.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(collection_slug)
                .to_string(),
            slug: collection_slug.to_string(),
            source: "magic_eden".to_string(),
            floor_price,
            currency: "SOL".to_string(),
            volume_24h: stats.get("volume24hr")
                .and_then(|v| v.as_f64())
                .map(|v| v / 1e9) // lamports → SOL
                .unwrap_or(0.0),
            total_volume: stats.get("volumeAll")
                .and_then(|v| v.as_f64())
                .map(|v| v / 1e9)
                .unwrap_or(0.0),
            market_cap: floor_price * stats.get("totalSupply")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            holder_count: 0, // Magic Eden API không cung cấp trực tiếp
            listed_count: stats.get("listedCount")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            total_supply: info_data.get("totalSupply")
                .and_then(|v| v.as_u64())
                .unwrap_or(10000),
            floor_price_7d_ago: None, // Cần call riêng
            raw_data: Some(stats),
        })
    }

    async fn fetch_nft(&self, contract: &str, token_id: &str) -> Result<NftInfo> {
        info!("Lấy thông tin NFT Magic Eden: {}", token_id);

        // Trên Solana, token_id chính là mint address
        let url = format!("{}/tokens/{}", self.base_url, token_id);
        let data = self.client.get_with_retry(&url, &self.api_headers(), 3).await?;

        // Parse attributes
        let mut traits = std::collections::HashMap::new();
        if let Some(attrs) = data.get("attributes").and_then(|v| v.as_array()) {
            for attr in attrs {
                if let (Some(trait_type), Some(value)) = (
                    attr.get("trait_type").and_then(|v| v.as_str()),
                    attr.get("value").and_then(|v| v.as_str()),
                ) {
                    traits.insert(trait_type.to_string(), value.to_string());
                }
            }
        }

        Ok(NftInfo {
            contract: contract.to_string(),
            token_id: token_id.to_string(),
            name: data.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            rarity_score: data.get("rarity")
                .and_then(|r| r.get("score"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            last_sale_price: data.get("price")
                .and_then(|v| v.as_f64())
                .map(|p| p / 1e9), // lamports → SOL
            traits,
            image_url: data.get("image")
                .and_then(|v| v.as_str())
                .map(String::from),
        })
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/collections/okay_bears/stats", self.base_url);
        match self.client.get_with_retry(&url, &self.api_headers(), 0).await {
            Ok(_) => {
                info!("Magic Eden API health check: OK");
                Ok(true)
            }
            Err(e) => {
                warn!("Magic Eden API health check failed: {}", e);
                Ok(false)
            }
        }
    }
}
