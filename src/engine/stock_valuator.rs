// === Stock Valuator - Phân tích định giá cổ phiếu ===
// Tính volatility, SMA/EMA, RSI, Bollinger Bands dựa trên lịch sử biến động

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::{info, warn};

use crate::config::DynamicConfigManager;
use crate::engine::{
    AssetType, ConfidenceLevel, PricingEngine, TrendDirection,
    ValuationRequest, ValuationResult, Recommendation,
};
use crate::engine::fundamental_analysis::{
    FundamentalAnalyzer, FundamentalConfig, FundamentalData,
    parse_fundamental_from_json,
};

/// Stock Valuator - engine phân tích cổ phiếu
pub struct StockValuator {
    /// Config manager (shared)
    config: Arc<DynamicConfigManager>,
}

/// Dữ liệu OHLCV cho một phiên giao dịch
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OhlcvBar {
    /// Thời điểm
    pub timestamp: DateTime<Utc>,
    /// Giá mở cửa
    pub open: f64,
    /// Giá cao nhất
    pub high: f64,
    /// Giá thấp nhất
    pub low: f64,
    /// Giá đóng cửa
    pub close: f64,
    /// Khối lượng
    pub volume: f64,
}

/// Kết quả phân tích kỹ thuật
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TechnicalAnalysis {
    /// Giá hiện tại
    pub current_price: f64,
    /// Biến động (standard deviation hàng năm)
    pub volatility: f64,
    /// SMA ngắn hạn (20 phiên)
    pub sma_20: f64,
    /// SMA dài hạn (50 phiên)
    pub sma_50: f64,
    /// EMA ngắn hạn (12 phiên)
    pub ema_12: f64,
    /// EMA dài hạn (26 phiên)
    pub ema_26: f64,
    /// RSI (14 phiên)
    pub rsi: f64,
    /// Bollinger Bands - upper
    pub bb_upper: f64,
    /// Bollinger Bands - middle (SMA20)
    pub bb_middle: f64,
    /// Bollinger Bands - lower
    pub bb_lower: f64,
    /// Vị trí trong Bollinger Bands (0=lower, 1=upper)
    pub bb_position: f64,
    /// % thay đổi giá so với phiên trước
    pub price_change_pct: f64,
    /// % thay đổi volume so với trung bình
    pub volume_change_pct: f64,
    /// Tín hiệu MA crossover: +1=buy, -1=sell, 0=neutral
    pub ma_signal: f64,
    /// ADX - sức mạnh xu hướng (0-100)
    pub trend_strength: f64,
}

impl StockValuator {
    /// Khởi tạo Stock Valuator
    pub fn new(config: Arc<DynamicConfigManager>) -> Self {
        info!("Khởi tạo StockValuator thành công");
        Self { config }
    }

    /// Tính Simple Moving Average
    pub fn calculate_sma(prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return 0.0;
        }
        let slice = &prices[prices.len() - period..];
        slice.iter().sum::<f64>() / period as f64
    }

    /// Tính Exponential Moving Average
    pub fn calculate_ema(prices: &[f64], period: usize) -> f64 {
        if prices.is_empty() || period == 0 {
            return 0.0;
        }
        if prices.len() < period {
            return Self::calculate_sma(prices, prices.len());
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = Self::calculate_sma(&prices[..period], period);

        for &price in &prices[period..] {
            ema = (price - ema) * multiplier + ema;
        }
        ema
    }

    /// Tính RSI (Relative Strength Index)
    pub fn calculate_rsi(prices: &[f64], period: usize) -> f64 {
        if prices.len() < period + 1 {
            return 50.0; // Giá trị trung lập khi không đủ dữ liệu
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        // Tính gain/loss cho mỗi phiên
        for i in 1..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change >= 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(change.abs());
            }
        }

        // Tính average gain/loss ban đầu
        let avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
        let avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return 100.0; // Toàn tăng
        }

        // Smoothed RSI
        let mut smooth_gain = avg_gain;
        let mut smooth_loss = avg_loss;

        for i in period..gains.len() {
            smooth_gain = (smooth_gain * (period as f64 - 1.0) + gains[i]) / period as f64;
            smooth_loss = (smooth_loss * (period as f64 - 1.0) + losses[i]) / period as f64;
        }

        if smooth_loss == 0.0 {
            return 100.0;
        }

        let rs = smooth_gain / smooth_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    /// Tính Historical Volatility (chuẩn hoá hàng năm)
    pub fn calculate_volatility(prices: &[f64]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }

        // Tính log returns
        let returns: Vec<f64> = prices.windows(2)
            .map(|w| (w[1] / w[0]).ln())
            .collect();

        if returns.is_empty() {
            return 0.0;
        }

        // Tính mean return
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;

        // Tính variance
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (returns.len() as f64 - 1.0);

        // Standard deviation * sqrt(252) cho annualized volatility
        let daily_vol = variance.sqrt();
        daily_vol * (252.0_f64).sqrt()
    }

    /// Tính Bollinger Bands
    pub fn calculate_bollinger_bands(prices: &[f64], period: usize, num_std: f64) -> (f64, f64, f64) {
        if prices.len() < period {
            return (0.0, 0.0, 0.0);
        }

        let slice = &prices[prices.len() - period..];
        let sma = slice.iter().sum::<f64>() / period as f64;

        let variance = slice.iter()
            .map(|p| (p - sma).powi(2))
            .sum::<f64>() / period as f64;
        let std_dev = variance.sqrt();

        let upper = sma + num_std * std_dev;
        let lower = sma - num_std * std_dev;

        (upper, sma, lower)
    }

    /// Phân tích kỹ thuật toàn diện từ dữ liệu OHLCV
    pub fn analyze(&self, bars: &[OhlcvBar]) -> TechnicalAnalysis {
        let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
        let volumes: Vec<f64> = bars.iter().map(|b| b.volume).collect();

        let current_price = *closes.last().unwrap_or(&0.0);

        // SMA
        let sma_20 = Self::calculate_sma(&closes, 20);
        let sma_50 = Self::calculate_sma(&closes, 50);

        // EMA
        let ema_12 = Self::calculate_ema(&closes, 12);
        let ema_26 = Self::calculate_ema(&closes, 26);

        // RSI
        let rsi = Self::calculate_rsi(&closes, 14);

        // Volatility
        let volatility = Self::calculate_volatility(&closes);

        // Bollinger Bands (20 phiên, 2 std)
        let (bb_upper, bb_middle, bb_lower) = Self::calculate_bollinger_bands(&closes, 20, 2.0);

        // Vị trí Bollinger
        let bb_position = if bb_upper > bb_lower {
            ((current_price - bb_lower) / (bb_upper - bb_lower)).clamp(0.0, 1.0)
        } else {
            0.5
        };

        // Price change %
        let price_change_pct = if closes.len() >= 2 {
            let prev = closes[closes.len() - 2];
            if prev > 0.0 { (current_price - prev) / prev } else { 0.0 }
        } else {
            0.0
        };

        // Volume change so với trung bình
        let avg_volume = if volumes.len() >= 20 {
            Self::calculate_sma(&volumes, 20)
        } else if !volumes.is_empty() {
            volumes.iter().sum::<f64>() / volumes.len() as f64
        } else {
            0.0
        };
        let current_volume = *volumes.last().unwrap_or(&0.0);
        let volume_change_pct = if avg_volume > 0.0 {
            (current_volume - avg_volume) / avg_volume
        } else {
            0.0
        };

        // MA crossover signal
        let ma_signal = if ema_12 > ema_26 && sma_20 > sma_50 {
            1.0 // Bullish
        } else if ema_12 < ema_26 && sma_20 < sma_50 {
            -1.0 // Bearish
        } else {
            0.0 // Neutral
        };

        // Trend strength (đơn giản hoá ADX)
        let trend_strength = {
            let price_vs_sma = ((current_price - sma_50) / sma_50 * 100.0).abs();
            price_vs_sma.min(100.0)
        };

        TechnicalAnalysis {
            current_price,
            volatility,
            sma_20,
            sma_50,
            ema_12,
            ema_26,
            rsi,
            bb_upper,
            bb_middle,
            bb_lower,
            bb_position,
            price_change_pct,
            volume_change_pct,
            ma_signal,
            trend_strength,
        }
    }

    /// Xác định xu hướng từ kết quả phân tích
    pub fn determine_trend(analysis: &TechnicalAnalysis) -> TrendDirection {
        let mut score: f64 = 0.0;

        // MA signal
        score += analysis.ma_signal * 2.0;

        // RSI
        if analysis.rsi > 70.0 { score += 1.0; }
        else if analysis.rsi < 30.0 { score -= 1.0; }

        // Price momentum
        if analysis.price_change_pct > 0.02 { score += 1.0; }
        else if analysis.price_change_pct < -0.02 { score -= 1.0; }

        // Bollinger position
        if analysis.bb_position > 0.8 { score += 0.5; }
        else if analysis.bb_position < 0.2 { score -= 0.5; }

        match score {
            s if s >= 3.0 => TrendDirection::StrongBullish,
            s if s >= 1.0 => TrendDirection::Bullish,
            s if s > -1.0 => TrendDirection::Neutral,
            s if s > -3.0 => TrendDirection::Bearish,
            _ => TrendDirection::StrongBearish,
        }
    }

    /// Định giá từ dữ liệu OHLCV
    pub fn valuate_from_bars(&self, symbol: &str, bars: &[OhlcvBar]) -> ValuationResult {
        let analysis = self.analyze(bars);
        let trend = Self::determine_trend(&analysis);
        let weights = &self.config.get_config().scoring.stock_weights;

        let mut result = ValuationResult::new(
            AssetType::Stock,
            symbol.to_string(),
            "yahoo_finance".to_string(),
        );
        result.currency = "USD".to_string();
        result.estimated_price = analysis.current_price;
        result.trend = trend;

        // === Tính điểm từng thuộc tính ===

        // Volatility (thấp = tốt, chuẩn hoá ngược)
        let vol_score = (1.0 - (analysis.volatility / 2.0).min(1.0)) * 100.0;
        result.attribute_scores.insert("volatility".to_string(), vol_score);

        // Price momentum
        let momentum_score = ((analysis.price_change_pct + 0.1) / 0.2 * 100.0).clamp(0.0, 100.0);
        result.attribute_scores.insert("price_momentum".to_string(), momentum_score);

        // Volume change
        let vol_change_score = ((analysis.volume_change_pct + 1.0) / 2.0 * 100.0).clamp(0.0, 100.0);
        result.attribute_scores.insert("volume_change".to_string(), vol_change_score);

        // RSI - trung lập là tốt nhất
        let rsi_score = (1.0 - ((analysis.rsi - 50.0) / 50.0).abs()) * 100.0;
        result.attribute_scores.insert("rsi".to_string(), rsi_score);

        // MA signal
        let ma_score = (analysis.ma_signal + 1.0) / 2.0 * 100.0;
        result.attribute_scores.insert("moving_avg_signal".to_string(), ma_score);

        // Bollinger position
        let bb_score = analysis.bb_position * 100.0;
        result.attribute_scores.insert("bollinger_position".to_string(), bb_score);

        // Trend strength
        result.attribute_scores.insert("trend_strength".to_string(), analysis.trend_strength);

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

        // Confidence dựa trên lượng dữ liệu
        result.confidence_pct = if bars.len() >= 200 {
            0.95
        } else if bars.len() >= 100 {
            0.80
        } else if bars.len() >= 50 {
            0.60
        } else if bars.len() >= 20 {
            0.40
        } else {
            0.20
        };
        result.calculate_confidence();

        // Lưu raw analysis
        result.raw_data = Some(serde_json::to_value(&analysis).unwrap_or_default());

        info!(
            "Stock valuation: {} | price={:.2} USD | score={:.1} | vol={:.1}% | RSI={:.1} | confidence={:.0}%",
            symbol, analysis.current_price, result.composite_score,
            analysis.volatility * 100.0, analysis.rsi,
            result.confidence_pct * 100.0
        );

        result
    }

    /// Định giá kết hợp technical + fundamental analysis
    pub fn valuate_with_fundamentals(
        &self,
        symbol: &str,
        bars: &[OhlcvBar],
        fundamental: &FundamentalData,
    ) -> ValuationResult {
        // Bước 1: Technical analysis
        let mut result = self.valuate_from_bars(symbol, bars);

        // Bước 2: Fundamental analysis
        let analyzer = FundamentalAnalyzer::with_defaults();
        let fund_result = analyzer.analyze(symbol, fundamental);

        // Bước 3: Gộp kết quả
        // Thêm fundamental scores vào attribute_scores
        for (key, metric) in &fund_result.metric_scores {
            let fund_key = format!("fundamental_{}", key);
            result.attribute_scores.insert(fund_key, metric.score);
        }

        // Kết hợp composite score: 40% technical + 60% fundamental
        let tech_score = result.composite_score;
        result.composite_score = tech_score * 0.40 + fund_result.composite_score * 0.60;

        // Gán khuyến nghị từ fundamental
        result.recommendation = Some(fund_result.recommendation);

        // Thêm notes
        result.notes.push(format!(
            "Giá trị nội tại ước tính: {:.2} {} (MoS: {:.1}%)",
            fund_result.intrinsic_value,
            result.currency,
            fund_result.margin_of_safety * 100.0
        ));
        result.notes.push(format!(
            "Trạng thái: {:?}",
            fund_result.valuation_status
        ));
        result.notes.extend(fund_result.analysis_notes.clone());

        // Lưu fundamental data vào raw_data
        if let Ok(fund_json) = serde_json::to_value(&fund_result) {
            let mut combined = serde_json::json!({});
            if let Some(tech_data) = &result.raw_data {
                combined["technical"] = tech_data.clone();
            }
            combined["fundamental"] = fund_json;
            result.raw_data = Some(combined);
        }

        info!(
            "Combined valuation {}: tech={:.1} + fund={:.1} = combined={:.1} | {:?}",
            symbol, tech_score, fund_result.composite_score,
            result.composite_score, result.recommendation
        );

        result
    }
}

#[async_trait]
impl PricingEngine for StockValuator {
    fn asset_type(&self) -> AssetType {
        AssetType::Stock
    }

    async fn valuate(&self, request: &ValuationRequest) -> anyhow::Result<ValuationResult> {
        info!("Bắt đầu phân tích cổ phiếu: {}", request.identifier);

        // Parse OHLCV từ additional_data
        if let Some(data) = &request.additional_data {
            // Kiểm tra có fundamental data không
            let has_fundamental = data.get("fundamental_data").is_some()
                || data.get("eps").is_some();

            let bars = parse_ohlcv_from_json(data)?;

            if has_fundamental {
                // Có cả technical + fundamental data
                let fund_data_json = data.get("fundamental_data")
                    .cloned()
                    .unwrap_or_else(|| data.clone());
                if let Ok(fund_data) = parse_fundamental_from_json(&fund_data_json) {
                    return Ok(self.valuate_with_fundamentals(
                        &request.identifier, &bars, &fund_data,
                    ));
                }
            }

            return Ok(self.valuate_from_bars(&request.identifier, &bars));
        }

        warn!("Không có dữ liệu OHLCV cho: {}. Cần gọi stock data fetcher.", request.identifier);
        let mut result = ValuationResult::new(
            AssetType::Stock,
            request.identifier.clone(),
            "none".to_string(),
        );
        result.confidence = ConfidenceLevel::VeryLow;
        result.notes.push("Không có dữ liệu OHLCV. Cần fetch từ Yahoo Finance.".to_string());
        Ok(result)
    }

    async fn valuate_batch(&self, requests: &[ValuationRequest]) -> Vec<anyhow::Result<ValuationResult>> {
        info!("Batch valuation cho {} stocks", requests.len());

        let mut handles = Vec::new();
        for req in requests {
            let req_clone = req.clone();
            let config_clone = self.config.clone();
            handles.push(tokio::spawn(async move {
                let valuator = StockValuator::new(config_clone);
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

/// Parse dữ liệu OHLCV từ JSON
pub fn parse_ohlcv_from_json(data: &serde_json::Value) -> anyhow::Result<Vec<OhlcvBar>> {
    // Hỗ trợ dạng array of bars
    if let Some(bars_array) = data.get("bars").and_then(|v| v.as_array()) {
        let bars: Vec<OhlcvBar> = bars_array.iter()
            .filter_map(|b| {
                Some(OhlcvBar {
                    timestamp: chrono::Utc::now(), // Placeholder
                    open: b.get("open")?.as_f64()?,
                    high: b.get("high")?.as_f64()?,
                    low: b.get("low")?.as_f64()?,
                    close: b.get("close")?.as_f64()?,
                    volume: b.get("volume")?.as_f64().unwrap_or(0.0),
                })
            })
            .collect();
        return Ok(bars);
    }

    // Hỗ trợ dạng tách riêng arrays (Yahoo Finance style)
    if let (Some(closes), Some(opens), Some(highs), Some(lows)) = (
        data.get("close").and_then(|v| v.as_array()),
        data.get("open").and_then(|v| v.as_array()),
        data.get("high").and_then(|v| v.as_array()),
        data.get("low").and_then(|v| v.as_array()),
    ) {
        let volumes = data.get("volume").and_then(|v| v.as_array());
        let bars: Vec<OhlcvBar> = (0..closes.len())
            .filter_map(|i| {
                Some(OhlcvBar {
                    timestamp: chrono::Utc::now(),
                    open: opens.get(i)?.as_f64()?,
                    high: highs.get(i)?.as_f64()?,
                    low: lows.get(i)?.as_f64()?,
                    close: closes.get(i)?.as_f64()?,
                    volume: volumes
                        .and_then(|v| v.get(i))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0),
                })
            })
            .collect();
        return Ok(bars);
    }

    Err(anyhow::anyhow!("Không thể parse dữ liệu OHLCV từ JSON"))
}
