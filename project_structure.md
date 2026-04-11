# Project Structure

```
Valuation_Pricing_Tools/
├── Cargo.toml                    # Dependencies & build config
├── Cargo.lock
├── config/
│   └── default.toml              # Cấu hình mặc định (server, proxy, scoring, etc.)
├── data/                         # SQLite database (auto-generated)
├── .env.example                  # Template biến môi trường
├── src/
│   ├── main.rs                   # Entry point: Tokio runtime, server init
│   ├── config/
│   │   ├── mod.rs                # AppConfig, DynamicConfigManager (ArcSwap)
│   │   ├── attributes.rs         # Dynamic attributes, WeightConfig
│   │   └── marketplace_profiles.rs # Per-marketplace profiles
│   ├── engine/
│   │   ├── mod.rs                # Core types: AssetType, Recommendation, ValuationResult
│   │   ├── nft_valuator.rs       # NFT pricing: rarity, floor, volume
│   │   ├── stock_valuator.rs     # Technical analysis: SMA, EMA, RSI, BB
│   │   ├── fundamental_analysis.rs # Fundamental: P/E, P/B, EPS, DCF
│   │   └── scoring.rs           # Scoring engine: weighted, normalization
│   ├── scrapers/
│   │   ├── mod.rs                # MarketplaceScraper trait, RateLimitedClient
│   │   ├── opensea.rs            # OpenSea API v2
│   │   ├── magic_eden.rs         # Magic Eden API
│   │   ├── stock_data.rs         # Yahoo Finance fetcher
│   │   └── proxy_pool.rs         # Proxy pool: rotation, health check
│   ├── crew_integration/
│   │   └── mod.rs                # CrewAI data processor, webhook handler
│   ├── api/
│   │   ├── mod.rs                # Axum router (13 endpoints)
│   │   ├── handlers.rs           # Request handlers
│   │   └── models.rs             # Request/Response structs
│   └── storage/
│       ├── mod.rs                # SQLite init, migrations
│       └── models.rs             # ValuationRepository
├── implementation_plan.md
├── README.md
├── project_structure.md
└── requirements.md
```
