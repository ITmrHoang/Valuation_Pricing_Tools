// === OpenSea Scraper - Tích hợp OpenSea API v2 ===
// Lấy collection stats, floor price, volume, NFT metadata

use async_trait::async_trait;
use anyhow::Result;
use tracing::{info, warn};

use super::{CollectionInfo, MarketplaceScraper, NftInfo, RateLimitedClient};

/// OpenSea API scraper
pub struct OpenSeaScraper {
    client: RateLimitedClient,
    base_url: String,
    api_key: String,
}

impl OpenSeaScraper {
    /// Khởi tạo OpenSea scraper
    pub fn new(base_url: &str, api_key: &str, rate_limit_ms: u64) -> Self {
        let client = RateLimitedClient::new(rate_limit_ms, "ValuationPricingTools/0.1.0", 30);
        info!("Khởi tạo OpenSea scraper: {}", base_url);

        Self {
            client,
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Headers chuẩn cho OpenSea API
    fn api_headers(&self) -> Vec<(String, String)> {
        let mut headers = vec![
            ("Accept".to_string(), "application/json".to_string()),
        ];
        if !self.api_key.is_empty() {
            headers.push(("X-API-KEY".to_string(), self.api_key.clone()));
        }
        headers
    }
}

#[async_trait]
impl MarketplaceScraper for OpenSeaScraper {
    fn name(&self) -> &str {
        "opensea"
    }

    async fn fetch_collection(&self, collection_slug: &str) -> Result<CollectionInfo> {
        info!("Lấy thông tin collection OpenSea: {}", collection_slug);

        let url = format!("{}/collections/{}", self.base_url, collection_slug);
        let data = self.client.get_with_retry(&url, &self.api_headers(), 3).await?;

        // Parse response từ OpenSea API v2
        let stats = data.get("stats").cloned().unwrap_or_default();

        Ok(CollectionInfo {
            name: data.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(collection_slug)
                .to_string(),
            slug: collection_slug.to_string(),
            source: "opensea".to_string(),
            floor_price: stats.get("floor_price")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            currency: "ETH".to_string(),
            volume_24h: stats.get("one_day_volume")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            total_volume: stats.get("total_volume")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            market_cap: stats.get("market_cap")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            holder_count: stats.get("num_owners")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            listed_count: stats.get("count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            total_supply: stats.get("total_supply")
                .and_then(|v| v.as_u64())
                .unwrap_or(10000),
            floor_price_7d_ago: stats.get("seven_day_average_price")
                .and_then(|v| v.as_f64()),
            raw_data: Some(data),
        })
    }

    async fn fetch_nft(&self, contract: &str, token_id: &str) -> Result<NftInfo> {
        info!("Lấy thông tin NFT OpenSea: {}#{}", contract, token_id);

        let url = format!(
            "{}/chain/ethereum/contract/{}/nfts/{}",
            self.base_url, contract, token_id
        );
        let data = self.client.get_with_retry(&url, &self.api_headers(), 3).await?;

        let nft = data.get("nft").unwrap_or(&data);

        // Parse traits
        let mut traits = std::collections::HashMap::new();
        if let Some(trait_list) = nft.get("traits").and_then(|v| v.as_array()) {
            for t in trait_list {
                if let (Some(trait_type), Some(value)) = (
                    t.get("trait_type").and_then(|v| v.as_str()),
                    t.get("value").and_then(|v| v.as_str()),
                ) {
                    traits.insert(trait_type.to_string(), value.to_string());
                }
            }
        }

        Ok(NftInfo {
            contract: contract.to_string(),
            token_id: token_id.to_string(),
            name: nft.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            rarity_score: nft.get("rarity")
                .and_then(|r| r.get("score"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            last_sale_price: nft.get("last_sale")
                .and_then(|s| s.get("total_price"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|p| p / 1e18), // Wei → ETH
            traits,
            image_url: nft.get("image_url")
                .and_then(|v| v.as_str())
                .map(String::from),
        })
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/collections/boredapeyachtclub", self.base_url);
        match self.client.get_with_retry(&url, &self.api_headers(), 0).await {
            Ok(_) => {
                info!("OpenSea API health check: OK");
                Ok(true)
            }
            Err(e) => {
                warn!("OpenSea API health check failed: {}", e);
                Ok(false)
            }
        }
    }
}
