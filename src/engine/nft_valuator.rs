// === NFT Valuator - Định giá NFT dựa trên multi-marketplace data ===
// Tính rarity score, floor price analysis, volume-weighted pricing

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, warn};

use crate::config::DynamicConfigManager;
use crate::engine::{
    AssetType, ConfidenceLevel, PricingEngine, TrendDirection,
    ValuationRequest, ValuationResult,
};
use crate::engine::scoring::ScoringEngine;

/// NFT Valuator - engine định giá NFT
pub struct NftValuator {
    /// Config manager (shared, lock-free read)
    config: Arc<DynamicConfigManager>,
    /// Scoring engine
    scoring: ScoringEngine,
}

/// Dữ liệu collection NFT từ marketplace
#[derive(Debug, Clone)]
pub struct CollectionData {
    /// Tên collection
    pub name: String,
    /// Giá sàn (native token)
    pub floor_price: f64,
    /// Khối lượng giao dịch 24h
    pub volume_24h: f64,
    /// Tổng khối lượng mọi thời đại
    pub total_volume: f64,
    /// Vốn hoá thị trường
    pub market_cap: f64,
    /// Số holder duy nhất
    pub holder_count: u64,
    /// Số item đang được list
    pub listed_count: u64,
    /// Tổng supply
    pub total_supply: u64,
    /// Giá sàn 7 ngày trước (để tính trend)
    pub floor_price_7d_ago: Option<f64>,
    /// Đơn vị tiền tệ (ETH, SOL, ...)
    pub currency: String,
}

/// Dữ liệu cá nhân NFT
#[derive(Debug, Clone)]
pub struct NftItemData {
    /// Token ID
    pub token_id: String,
    /// Tên item
    pub name: String,
    /// Rarity score (0-100)
    pub rarity_score: f64,
    /// Giá bán cuối cùng
    pub last_sale_price: Option<f64>,
    /// Traits/Attributes
    pub traits: HashMap<String, String>,
}

impl NftValuator {
    /// Khởi tạo NFT Valuator
    pub fn new(config: Arc<DynamicConfigManager>) -> Self {
        let scoring = ScoringEngine::new(config.clone());
        info!("Khởi tạo NftValuator thành công");
        Self { config, scoring }
    }

    /// Tính điểm rarity cho NFT dựa trên traits
    pub fn calculate_rarity_score(&self, item: &NftItemData, total_supply: u64) -> f64 {
        if item.traits.is_empty() || total_supply == 0 {
            return 50.0; // Trả về trung bình khi không có dữ liệu
        }

        // Tính rarity dựa trên số lượng trait unique
        // Mỗi trait hiếm → điểm cao hơn
        let trait_count = item.traits.len() as f64;
        let base_score = (trait_count / 10.0).min(1.0) * 100.0;

        // Nếu có rarity score sẵn, ưu tiên dùng
        if item.rarity_score > 0.0 {
            return item.rarity_score;
        }

        base_score
    }

    /// Phân tích xu hướng giá collection
    pub fn analyze_trend(&self, collection: &CollectionData) -> TrendDirection {
        match collection.floor_price_7d_ago {
            Some(price_7d) if price_7d > 0.0 => {
                let change_pct = (collection.floor_price - price_7d) / price_7d;
                match change_pct {
                    p if p > 0.20 => TrendDirection::StrongBullish,
                    p if p > 0.05 => TrendDirection::Bullish,
                    p if p > -0.05 => TrendDirection::Neutral,
                    p if p > -0.20 => TrendDirection::Bearish,
                    _ => TrendDirection::StrongBearish,
                }
            }
            _ => TrendDirection::Neutral,
        }
    }

    /// Tính liquidity score dựa trên volume và listed count
    pub fn calculate_liquidity(&self, collection: &CollectionData) -> f64 {
        if collection.total_supply == 0 {
            return 0.0;
        }

        // Tỷ lệ item được list = thanh khoản cơ bản
        let list_ratio = collection.listed_count as f64 / collection.total_supply as f64;

        // Volume 24h so với market cap = thanh khoản giao dịch
        let volume_ratio = if collection.market_cap > 0.0 {
            (collection.volume_24h / collection.market_cap).min(1.0)
        } else {
            0.0
        };

        // Kết hợp 2 yếu tố
        (list_ratio * 0.4 + volume_ratio * 0.6).min(1.0)
    }

    /// Định giá NFT dựa trên dữ liệu collection
    pub fn valuate_from_collection_data(
        &self,
        collection: &CollectionData,
        item: Option<&NftItemData>,
    ) -> ValuationResult {
        let mut result = ValuationResult::new(
            AssetType::Nft,
            collection.name.clone(),
            "multi_marketplace".to_string(),
        );

        result.currency = collection.currency.clone();

        // === Tính điểm từng thuộc tính ===
        let weights = &self.config.get_config().scoring.nft_weights;

        // 1. Rarity score
        let rarity = match item {
            Some(nft) => self.calculate_rarity_score(nft, collection.total_supply),
            None => 50.0,
        };
        result.attribute_scores.insert("rarity".to_string(), rarity);

        // 2. Floor price (chuẩn hoá log scale)
        let floor_score = if collection.floor_price > 0.0 {
            (collection.floor_price.ln() + 5.0).max(0.0).min(100.0)
        } else {
            0.0
        };
        result.attribute_scores.insert("floor_price".to_string(), floor_score);

        // 3. Volume 24h (chuẩn hoá)
        let volume_score = if collection.volume_24h > 0.0 {
            (collection.volume_24h.ln() * 10.0).max(0.0).min(100.0)
        } else {
            0.0
        };
        result.attribute_scores.insert("volume_24h".to_string(), volume_score);

        // 4. Market cap score
        let mcap_score = if collection.market_cap > 0.0 {
            (collection.market_cap.ln() * 8.0).max(0.0).min(100.0)
        } else {
            0.0
        };
        result.attribute_scores.insert("market_cap".to_string(), mcap_score);

        // 5. Trend score
        let trend = self.analyze_trend(collection);
        let trend_score = match &trend {
            TrendDirection::StrongBullish => 100.0,
            TrendDirection::Bullish => 75.0,
            TrendDirection::Neutral => 50.0,
            TrendDirection::Bearish => 25.0,
            TrendDirection::StrongBearish => 0.0,
        };
        result.attribute_scores.insert("trend".to_string(), trend_score);
        result.trend = trend;

        // 6. Liquidity
        let liquidity = self.calculate_liquidity(collection) * 100.0;
        result.attribute_scores.insert("liquidity".to_string(), liquidity);

        // 7. Holder count (chuẩn hoá)
        let holder_score = if collection.holder_count > 0 {
            ((collection.holder_count as f64).ln() * 12.0).max(0.0).min(100.0)
        } else {
            0.0
        };
        result.attribute_scores.insert("holder_count".to_string(), holder_score);

        // === Tính composite score ===
        let mut composite = 0.0;
        let mut total_weight = 0.0;
        for (key, &score) in &result.attribute_scores {
            let w = weights.get_weight(key);
            composite += score * w;
            total_weight += w;
        }
        if total_weight > 0.0 {
            result.composite_score = composite / total_weight;
        }

        // === Ước tính giá ===
        // Dựa trên floor price điều chỉnh bởi rarity
        let rarity_multiplier = if rarity > 80.0 {
            2.0 + (rarity - 80.0) / 20.0
        } else if rarity > 50.0 {
            1.0 + (rarity - 50.0) / 60.0
        } else {
            0.5 + rarity / 100.0
        };
        result.estimated_price = collection.floor_price * rarity_multiplier;

        // === Tính confidence ===
        let mut data_completeness = 0.0;
        let total_fields = 7.0;
        if collection.floor_price > 0.0 { data_completeness += 1.0; }
        if collection.volume_24h > 0.0 { data_completeness += 1.0; }
        if collection.market_cap > 0.0 { data_completeness += 1.0; }
        if collection.holder_count > 0 { data_completeness += 1.0; }
        if collection.floor_price_7d_ago.is_some() { data_completeness += 1.0; }
        if collection.listed_count > 0 { data_completeness += 1.0; }
        if item.is_some() { data_completeness += 1.0; }

        result.confidence_pct = data_completeness / total_fields;
        result.calculate_confidence();

        info!(
            "NFT valuation: {} | score={:.1} | price={:.4} {} | confidence={:.0}%",
            collection.name, result.composite_score,
            result.estimated_price, result.currency,
            result.confidence_pct * 100.0
        );

        result
    }
}

#[async_trait]
impl PricingEngine for NftValuator {
    fn asset_type(&self) -> AssetType {
        AssetType::Nft
    }

    async fn valuate(&self, request: &ValuationRequest) -> anyhow::Result<ValuationResult> {
        info!("Bắt đầu định giá NFT: {}", request.identifier);

        // Trong trường hợp có additional_data từ CrewAI hoặc scraper
        if let Some(data) = &request.additional_data {
            let collection = parse_collection_from_json(data)?;
            return Ok(self.valuate_from_collection_data(&collection, None));
        }

        // Nếu không có dữ liệu, trả về kết quả trống
        warn!("Không có dữ liệu cho NFT: {}. Cần gọi scraper trước.", request.identifier);
        let mut result = ValuationResult::new(
            AssetType::Nft,
            request.identifier.clone(),
            "none".to_string(),
        );
        result.confidence = ConfidenceLevel::VeryLow;
        result.notes.push("Không có dữ liệu marketplace. Cần chạy scraper.".to_string());
        Ok(result)
    }

    async fn valuate_batch(&self, requests: &[ValuationRequest]) -> Vec<anyhow::Result<ValuationResult>> {
        info!("Batch valuation cho {} NFTs", requests.len());

        // Sử dụng tokio::spawn cho parallel processing
        let mut handles = Vec::new();
        for req in requests {
            let req_clone = req.clone();
            let config_clone = self.config.clone();
            handles.push(tokio::spawn(async move {
                let valuator = NftValuator::new(config_clone);
                valuator.valuate(&req_clone).await
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(anyhow::anyhow!("Task join error: {}", e))),
            }
        }
        results
    }
}

/// Parse dữ liệu collection từ JSON (từ CrewAI hoặc scraper)
fn parse_collection_from_json(data: &serde_json::Value) -> anyhow::Result<CollectionData> {
    Ok(CollectionData {
        name: data.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        floor_price: data.get("floor_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        volume_24h: data.get("volume_24h")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        total_volume: data.get("total_volume")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        market_cap: data.get("market_cap")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        holder_count: data.get("holder_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        listed_count: data.get("listed_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        total_supply: data.get("total_supply")
            .and_then(|v| v.as_u64())
            .unwrap_or(10000),
        floor_price_7d_ago: data.get("floor_price_7d_ago")
            .and_then(|v| v.as_f64()),
        currency: data.get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("ETH")
            .to_string(),
    })
}
