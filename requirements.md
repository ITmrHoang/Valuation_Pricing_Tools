# Requirements - AI Config Pricing Engine

| `mod.rs` | Core types: AssetType, TrendDirection, Recommendation, ValuationResult, PricingEngine trait |
| `nft_valuator.rs` | Định giá NFT: rarity score, floor price analysis, volume-weighted pricing |
| `stock_valuator.rs` | Technical analysis: SMA, EMA, RSI, Bollinger Bands, Volatility |
| `fundamental_analysis.rs` | **[NEW]** Fundamental analysis: P/E, P/B, EPS growth, DCF, khuyến nghị mua/bán |
## v0.1.0 (2026-04-10) — Core Foundation

### R-001: NFT Valuation Engine
- Định giá NFT dựa trên multi-marketplace data (OpenSea, Magic Eden, Blur)
- Dynamic attributes: rarity, floor price, volume, market cap, trend, liquidity
- Weighted scoring với configurable trọng số

### R-002: Stock Technical Analysis
- Tính toán SMA, EMA, RSI, Bollinger Bands, Historical Volatility
- OHLCV data processing từ Yahoo Finance
- Trend detection (StrongBullish → StrongBearish)

### R-003: Dynamic Config System
- Hot-reload config via ArcSwap (lock-free read)
- TOML configuration file
- Runtime update qua REST API

### R-004: REST API
- Axum-based, async, multi-threaded
- CRUD config, valuation endpoints, history
- CrewAI webhook integration

### R-005: Data Persistence
- SQLite storage (valuation_history, config_snapshots, price_data, crew_webhook_data)
- Repository pattern

---

## v0.2.0 (2026-04-11) — Fundamental Analysis & Proxy

### R-006: Stock Fundamental Analysis
- Phân tích P/E ratio vs ngành
- Phân tích P/B ratio (giá trị sổ sách)
- EPS growth analysis
- Dividend yield & payout ratio analysis
- DCF (Discounted Cash Flow) valuation — intrinsic value estimation
- Stability score (D/E, ROE, revenue growth)
- **Khuyến nghị**: StrongBuy/Buy/Hold/Sell/StrongSell
- **Margin of Safety**: % chênh lệch giữa giá thị trường và giá trị nội tại
- API endpoint: `POST /api/v1/valuate/stock/fundamental`

### R-007: Proxy Pool Multi-threaded
- Round-robin proxy rotation
- Health check tự động (mark dead/alive)
- Failover khi proxy die
- Hỗ trợ HTTP/HTTPS/SOCKS5 với authentication
- Tích hợp vào RateLimitedClient
- Config qua `config/default.toml` section `[proxy]`
