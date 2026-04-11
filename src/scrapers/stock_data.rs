// === Stock Data Fetcher - Lấy dữ liệu cổ phiếu từ Yahoo Finance ===
// Historical OHLCV, real-time quotes

use anyhow::Result;
use chrono::{DateTime, Utc, TimeZone, Duration};
use tracing::{info, warn};

use super::RateLimitedClient;
use crate::engine::stock_valuator::OhlcvBar;

/// Stock data fetcher từ Yahoo Finance
pub struct StockDataFetcher {
    client: RateLimitedClient,
    base_url: String,
}

/// Kết quả lấy dữ liệu stock
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StockQuote {
    /// Mã cổ phiếu
    pub symbol: String,
    /// Giá hiện tại
    pub current_price: f64,
    /// Thay đổi giá trong phiên
    pub price_change: f64,
    /// % thay đổi
    pub change_percent: f64,
    /// Khối lượng giao dịch
    pub volume: f64,
    /// Giá cao nhất trong phiên
    pub day_high: f64,
    /// Giá thấp nhất trong phiên
    pub day_low: f64,
    /// Vốn hoá thị trường
    pub market_cap: Option<f64>,
    /// Dữ liệu lịch sử OHLCV
    pub historical_bars: Vec<OhlcvBar>,
}

impl StockDataFetcher {
    /// Khởi tạo fetcher
    pub fn new(base_url: &str, rate_limit_ms: u64) -> Self {
        let client = RateLimitedClient::new(rate_limit_ms, "ValuationPricingTools/0.1.0", 30);
        info!("Khởi tạo StockDataFetcher: {}", base_url);

        Self {
            client,
            base_url: base_url.to_string(),
        }
    }

    /// Lấy dữ liệu lịch sử OHLCV cho một mã cổ phiếu
    pub async fn fetch_historical(
        &self,
        symbol: &str,
        days: u32,
        interval: &str,
    ) -> Result<Vec<OhlcvBar>> {
        info!("Lấy dữ liệu lịch sử {} ngày cho: {} (interval: {})", days, symbol, interval);

        // Tính range thời gian
        let now = Utc::now().timestamp();
        let period_start = now - (days as i64 * 86400);

        let url = format!(
            "{}/chart/{}?period1={}&period2={}&interval={}&includePrePost=false",
            self.base_url, symbol, period_start, now, interval
        );

        let headers = vec![
            ("Accept".to_string(), "application/json".to_string()),
        ];

        let data = self.client.get_with_retry(&url, &headers, 3).await?;

        // Parse Yahoo Finance response
        let bars = parse_yahoo_chart_response(&data, symbol)?;

        info!("Đã lấy {} bars cho {}", bars.len(), symbol);
        Ok(bars)
    }

    /// Lấy quote hiện tại cho một mã cổ phiếu
    pub async fn fetch_quote(&self, symbol: &str) -> Result<StockQuote> {
        info!("Lấy quote hiện tại cho: {}", symbol);

        let url = format!(
            "{}/quote?symbols={}",
            self.base_url.replace("/chart", ""), symbol
        );

        let headers = vec![
            ("Accept".to_string(), "application/json".to_string()),
        ];

        let data = self.client.get_with_retry(&url, &headers, 3).await?;

        // Parse response
        let quote_data = data.get("quoteResponse")
            .and_then(|r| r.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|a| a.first())
            .ok_or_else(|| anyhow::anyhow!("Không tìm thấy dữ liệu quote cho {}", symbol))?;

        Ok(StockQuote {
            symbol: symbol.to_string(),
            current_price: quote_data.get("regularMarketPrice")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            price_change: quote_data.get("regularMarketChange")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            change_percent: quote_data.get("regularMarketChangePercent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            volume: quote_data.get("regularMarketVolume")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            day_high: quote_data.get("regularMarketDayHigh")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            day_low: quote_data.get("regularMarketDayLow")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            market_cap: quote_data.get("marketCap")
                .and_then(|v| v.as_f64()),
            historical_bars: Vec::new(),
        })
    }

    /// Lấy dữ liệu đầy đủ (quote + history)
    pub async fn fetch_full(&self, symbol: &str, history_days: u32) -> Result<StockQuote> {
        let mut quote = self.fetch_quote(symbol).await
            .unwrap_or_else(|e| {
                warn!("Không lấy được quote {}: {}. Sử dụng dữ liệu lịch sử.", symbol, e);
                StockQuote {
                    symbol: symbol.to_string(),
                    current_price: 0.0,
                    price_change: 0.0,
                    change_percent: 0.0,
                    volume: 0.0,
                    day_high: 0.0,
                    day_low: 0.0,
                    market_cap: None,
                    historical_bars: Vec::new(),
                }
            });

        let bars = self.fetch_historical(symbol, history_days, "1d").await?;

        // Cập nhật current_price từ historical nếu quote fail
        if quote.current_price == 0.0 {
            if let Some(last_bar) = bars.last() {
                quote.current_price = last_bar.close;
            }
        }

        quote.historical_bars = bars;
        Ok(quote)
    }
}

/// Parse response từ Yahoo Finance chart API
fn parse_yahoo_chart_response(data: &serde_json::Value, symbol: &str) -> Result<Vec<OhlcvBar>> {
    let chart = data.get("chart")
        .and_then(|c| c.get("result"))
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| anyhow::anyhow!("Không có dữ liệu chart cho {}", symbol))?;

    let timestamps = chart.get("timestamp")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Không tìm thấy timestamps"))?;

    let indicators = chart.get("indicators")
        .and_then(|i| i.get("quote"))
        .and_then(|q| q.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| anyhow::anyhow!("Không tìm thấy OHLCV data"))?;

    let opens = indicators.get("open").and_then(|v| v.as_array());
    let highs = indicators.get("high").and_then(|v| v.as_array());
    let lows = indicators.get("low").and_then(|v| v.as_array());
    let closes = indicators.get("close").and_then(|v| v.as_array());
    let volumes = indicators.get("volume").and_then(|v| v.as_array());

    let mut bars = Vec::new();

    for i in 0..timestamps.len() {
        let ts = timestamps[i].as_i64().unwrap_or(0);
        let timestamp = Utc.timestamp_opt(ts, 0)
            .single()
            .unwrap_or_else(Utc::now);

        let open = opens.and_then(|a| a.get(i)).and_then(|v| v.as_f64());
        let high = highs.and_then(|a| a.get(i)).and_then(|v| v.as_f64());
        let low = lows.and_then(|a| a.get(i)).and_then(|v| v.as_f64());
        let close = closes.and_then(|a| a.get(i)).and_then(|v| v.as_f64());
        let volume = volumes.and_then(|a| a.get(i)).and_then(|v| v.as_f64()).unwrap_or(0.0);

        // Bỏ qua bar nếu thiếu dữ liệu chính
        if let (Some(o), Some(h), Some(l), Some(c)) = (open, high, low, close) {
            bars.push(OhlcvBar {
                timestamp,
                open: o,
                high: h,
                low: l,
                close: c,
                volume,
            });
        }
    }

    Ok(bars)
}
