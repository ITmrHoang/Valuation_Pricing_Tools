# Project Structure — Valuation Pricing Tools v1.0

## Nguyên Tắc Tổ Chức

```
CORE  = Tính năng dùng CHUNG (infrastructure, shared services)
        → Mọi module đều dùng lại, không duplicate
        
MODULE = Logic RIÊNG từng domain (stock_vn, msu_game, ...)
         → Phát triển độc lập, chỉ phụ thuộc vào Core
         → Mỗi module có DB riêng, crawler riêng, valuation riêng
```

## Tổng Quan

```
Valuation_Pricing_Tools/           # Workspace root
│
├── Cargo.toml                     # Workspace manifest (members)
│
├── core/                          # 🔧 CORE — Tính năng dùng chung
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                # Entry point, khởi tạo system
│       │
│       ├── config/                # [CHUNG] Config Manager
│       │   ├── mod.rs             # AppConfig, load TOML
│       │   └── dynamic.rs         # ArcSwap hot-reload runtime
│       │
│       ├── registry/              # [CHUNG] Module Registry
│       │   └── mod.rs             # DashMap cache + SQLite persist
│       │                          # Load/register/toggle modules
│       │
│       ├── runtime/               # [CHUNG] WASM Runtime
│       │   └── mod.rs             # Wasmtime engine, module cache
│       │                          # Pre-compile .wasm → native code
│       │
│       ├── crawler/               # [CHUNG] Crawl Infrastructure
│       │   ├── mod.rs             # CrawlService trait
│       │   ├── http_client.rs     # reqwest + proxy rotation
│       │   │                      # Giả lập headers, cookie
│       │   └── playwright.rs      # Playwright subprocess wrapper
│       │                          # Gọi Node.js scripts từ Rust
│       │
│       ├── proxy/                 # [CHUNG] Proxy Pool Manager
│       │   └── mod.rs             # 2 pools: API proxy + Browser proxy
│       │                          # Round-robin, health check, failover
│       │
│       ├── scheduler/             # [CHUNG] Task Scheduler
│       │   └── mod.rs             # 2 chế độ:
│       │                          #   - Batch: chạy theo cron (stock)
│       │                          #   - Stream: loop liên tục (NFT)
│       │
│       ├── api/                   # [CHUNG] REST API Gateway
│       │   ├── mod.rs             # Axum router, middleware
│       │   ├── handlers.rs        # Request handlers
│       │   └── models.rs          # Request/Response structs
│       │
│       ├── storage/               # [CHUNG] Database Helpers
│       │   └── mod.rs             # SQLite connection pool
│       │                          # Migration runner
│       │                          # Core DB (registry, config)
│       │
│       ├── state/                 # 🆕 [CHUNG] State Manager
│       │   └── mod.rs             # Lưu trạng thái runtime:
│       │                          #   - Module nào đang bật/tắt
│       │                          #   - Service nào đang chạy
│       │                          #   - Lần crawl cuối cùng
│       │                          #   - Error count
│       │                          # Persist vào core.db
│       │                          # Khi startup:
│       │                          #   1. Load modules.toml (default)
│       │                          #   2. Load core.db:module_state
│       │                          #      (override runtime state)
│       │                          #   3. Merge → final state
│       │
│       └── common/                # [CHUNG] Types & Utils dùng chung
│           ├── mod.rs
│           ├── types.rs           # Score, Recommendation, Trend,
│           │                      # ConfidenceLevel, ValuationResult
│           ├── errors.rs          # Error types thống nhất
│           └── utils.rs           # Date format, JSON helpers,
│                                  # Price conversion, math utils
│
├── modules/                       # 📦 MODULES — Logic riêng từng domain
│   │
│   ├── stock_vn/                  # 📈 Module: Cổ phiếu Việt Nam
│   │   ├── Cargo.toml             # Dependencies: core (path dep)
│   │   └── src/
│   │       ├── lib.rs             # Public API: crawl(), valuate()
│   │       │
│   │       ├── crawler/           # [RIÊNG] Crawl data cổ phiếu
│   │       │   ├── mod.rs         # CrawlerManager cho stock
│   │       │   ├── simplize.rs    # Reverse-engineer Simplize.vn
│   │       │   │                  #   - Tổng quan (free)
│   │       │   │                  #   - BCTC (cần login → Playwright)
│   │       │   ├── scanner.rs     # 🆕 SCANNER: Quét TOÀN BỘ mã VN
│   │       │   │                  #   Crawl từ Simplize danh sách
│   │       │   │                  #   tất cả cổ phiếu HOSE/HNX/UPCOM
│   │       │   │                  #   Lấy: giá, P/B, P/E, cổ tức,
│   │       │   │                  #   vốn hóa, EPS, tăng trưởng
│   │       │   │                  #   → Lưu vào DB → chạy screener
│   │       │   └── yahoo.rs       # Yahoo Finance API (backup)
│   │       │                      #   - OHLCV: /v8/finance/chart/VNM.VN
│   │       │                      #   - Quote: /v7/finance/quote/VNM.VN
│   │       │
│   │       ├── valuation/         # [RIÊNG] Phân tích & Định giá
│   │       │   ├── mod.rs         # Orchestrator: tech + fund → score
│   │       │   ├── technical.rs   # 20+ chỉ báo kỹ thuật:
│   │       │   │                  #   Trend: SMA, EMA, MACD, ADX, Ichimoku
│   │       │   │                  #   Oscillator: RSI, Stochastic, CCI, W%R
│   │       │   │                  #   Volatility: BB, ATR, HistVol
│   │       │   │                  #   Volume: OBV, VWAP, MFI, VolChange
│   │       │   │                  #   Signal: Golden/Death Cross, ROC, SAR
│   │       │   ├── fundamental.rs # Phân tích cơ bản:
│   │       │   │                  #   Định giá: P/E, P/B, PEG, EV/EBITDA
│   │       │   │                  #   Tăng trưởng: EPS Growth, Rev Growth
│   │       │   │                  #   Sức khỏe: D/E, ROE, Current Ratio
│   │       │   │                  #   Thu nhập: Div Yield, Payout Ratio
│   │       │   │                  #   DCF: Intrinsic Value, Margin of Safety
│   │       │   └── recommendation.rs  # Tổng hợp → StrongBuy/Sell
│   │       │                          # Final = 40% Tech + 60% Fund
│   │       │
│   │       ├── screener/          # 🆕 [RIÊNG] Stock Screener
│   │       │   ├── mod.rs         # Orchestrator scanner flow
│   │       │   └── value_screener.rs  # Lọc + Xếp hạng cổ phiếu
│   │       │                      #   giá trị đầu tư dài hạn:
│   │       │                      #
│   │       │                      #   Input: Toàn bộ mã VN từ scanner
│   │       │                      #   Tiêu chí lọc:
│   │       │                      #   1. P/B < 1.5 (giá < giá trị tài sản)
│   │       │                      #   2. Cổ tức ổn định ≥3 năm liên tục
│   │       │                      #   3. Dividend Yield > lãi suất tiết kiệm
│   │       │                      #   4. Tăng trưởng EPS dương
│   │       │                      #   5. D/E < 1.5 (ít nợ)
│   │       │                      #   6. ROE > 12%
│   │       │                      #   Output: Top N cổ phiếu đáng đầu tư
│   │       │                      #   + Value Score + Dividend Score
│   │       │
│   │       ├── models/            # [RIÊNG] Data models cho stock
│   │       │   └── mod.rs         # StockData, OHLCV, FundamentalData,
│   │       │                      # ScanResult, ScreenerResult
│   │       │
│   │       └── storage/           # [RIÊNG] DB schema cho stock
│   │           └── mod.rs         # stock_vn.db tables:
│   │                              #   - price_history (OHLCV)
│   │                              #   - fundamental_data
│   │                              #   - valuation_results
│   │                              #   - watchlist
│   │                              #   - 🆕 stock_universe (tất cả mã VN)
│   │                              #   - 🆕 dividend_history (lịch sử cổ tức)
│   │                              #   - 🆕 screener_results (kết quả lọc)
│   │
│   └── msu_game/                  # 🎮 Module: MapleStory Universe
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs             # Public API: crawl(), valuate(), trade()
│           │
│           ├── crawler/           # [RIÊNG] Crawl MSU marketplace
│           │   ├── mod.rs         # CrawlerManager cho MSU
│           │   └── marketplace.rs # Reverse-engineer msu.io API
│           │                      #   - /marketplace/nft → items
│           │                      #   - /marketplace/ft → consumables
│           │                      #   - /marketplace/character → chars
│           │                      #   - Recent sales, top traders
│           │
│           ├── valuation/         # [RIÊNG] Định giá NFT
│           │   ├── mod.rs         # Orchestrator: tổng hợp 5 tầng
│           │   ├── enhancement.rs # Cấp độ & Star Force (20%)
│           │   │                  #   Chi phí nâng cấp tích lũy
│           │   │                  #   Success rate × cost = expected value
│           │   ├── rarity.rs      # Độ hiếm (25%)
│           │   │                  #   Tier, Trait rarity, Supply,
│           │   │                  #   Potential tier
│           │   ├── stats.rs       # 🆕 Chỉ số / Stats (20%)
│           │   │                  #   Total stats vs max possible
│           │   │                  #   Percentile ranking
│           │   ├── attributes.rs  # 🆕 Thuộc tính đặc biệt (15%)
│           │   │                  #   Potential lines, Set bonus,
│           │   │                  #   Class restriction, Soul/Flame
│           │   ├── pricing.rs     # Yếu tố thị trường (20%)
│           │   │                  #   Floor price, Avg/Median sales,
│           │   │                  #   Volume 7d, Price trend
│           │   ├── anti_pump.rs   # 🆕 Chống bơm giá:
│           │   │                  #   D1: Price Spike (3σ detection)
│           │   │                  #   D2: Wash Trading (cycle detect)
│           │   │                  #   D3: Volume Anomaly
│           │   │                  #   D4: Price Reversion
│           │   │                  #   D5: Wallet Clustering
│           │   │                  #   Benford's Law check
│           │   ├── safe_price.rs  # 🆕 Safe Price Calculation:
│           │   │                  #   Loại bỏ giao dịch thao túng
│           │   │                  #   Dùng MEDIAN thay vì MEAN
│           │   │                  #   Confidence level assessment
│           │   └── deal_finder.rs # 🔥 Flipping Engine:
│           │                      #   Deal Score + Anti-pump check
│           │                      #   Auto buy nếu score ≥ 40%
│           │                      #   + HighConfidence + clean wallet
│           │
│           ├── trade/             # [RIÊNG] Auto buy/sell
│           │   ├── mod.rs
│           │   └── browser_trader.rs  # Playwright auto mua/bán
│           │                          # trên msu.io marketplace
│           │
│           ├── models/            # [RIÊNG] Data models cho MSU
│           │   └── mod.rs         # NftItem, Character, Sale, Deal
│           │
│           └── storage/           # [RIÊNG] DB schema cho MSU
│               └── mod.rs         # msu.db tables:
│                                  #   - nft_items
│                                  #   - ft_items
│                                  #   - characters
│                                  #   - sales_history
│                                  #   - deals (flipping tracker)
│                                  #   - portfolio (items đang hold)
│
├── playwright/                    # 🎭 Playwright Scripts (Node.js)
│   ├── package.json               # Dependencies: playwright, stealth
│   ├── scripts/
│   │   ├── simplize_crawler.js    # Crawl Simplize.vn
│   │   │                          #   Cách lấy data:
│   │   │                          #   1. page.on('response') → bắt API
│   │   │                          #      nội bộ website gọi (TỐT NHẤT)
│   │   │                          #   2. page.evaluate() → inject JS
│   │   │                          #      vào DOM, đọc dữ liệu text
│   │   │                          #   3. page.route() → intercept +
│   │   │                          #      modify request/response
│   │   ├── msu_crawler.js         # Crawl MSU marketplace
│   │   │                          #   Bắt API: msu.io gọi internal API
│   │   │                          #   khi load marketplace → intercept
│   │   │                          #   response → lấy JSON items/prices
│   │   └── msu_trader.js          # Auto buy/sell trên MSU
│   │                              #   Navigate → Click Buy → Confirm
│   ├── stealth/
│   │   └── config.js              # Anti-detect: fake fingerprint,
│   │                              # random delays, human behavior
│   └── sessions/                  # 🔐 Lưu session/cookie browser
│       ├── simplize_session.json  # Cookie đăng nhập Simplize
│       └── msu_session.json       # Cookie/wallet session MSU
│
├── web/                           # 🌐 Web UI
│   ├── index.html                 # Dashboard tổng quan
│   ├── admin.html                 # 🆕 ADMIN: Quản lý modules
│   │                              #   - Bật/tắt module (stock_vn, msu)
│   │                              #   - Bật/tắt service (crawler/val/trade)
│   │                              #   - Xem trạng thái (running/stopped)
│   │                              #   - Xem log realtime
│   │                              #   - Start/Stop từng service
│   ├── stock.html                 # Stock analysis view
│   ├── deals.html                 # NFT deals view
│   ├── css/
│   │   └── style.css
│   └── js/
│       ├── app.js                 # Logic chung
│       └── admin.js               # 🆕 Logic trang admin
│
├── compiled/                      # 📦 WASM compiled outputs
│   ├── stock_vn.wasm
│   └── msu_game.wasm
│
├── data/                          # 💾 SQLite databases
│   ├── core.db                    # Registry, config snapshots,
│   │                              # MODULE STATE (trạng thái runtime)
│   │                              # Khi restart → load state từ đây
│   ├── stock_vn.db                # Giá cổ phiếu, BCTC, valuations
│   └── msu.db                     # Items, sales, deals, portfolio
│
├── config/                        # ⚙️ Configuration files
│   ├── core.toml                  # Server, proxy pools, browser
│   └── modules.toml               # Bật/tắt modules + services
│
├── docs/                          # 📄 Documentation
│   └── usage.md                   # Hướng dẫn sử dụng
│
├── implementation_plan.md         # Kế hoạch triển khai
├── project_structure.md           # File này
├── requirements.md                # Yêu cầu nghiệp vụ
└── README.md                      # Tổng quan project
```

---

## Phân Rõ: Core vs Module

### Core cung cấp GÌ cho Modules?

| Service | Core cung cấp | Module sử dụng |
|---|---|---|
| **HTTP Client** | `reqwest` + proxy rotation + giả lập headers | Module gọi `core::crawler::http_get(url)` |
| **Playwright** | Subprocess wrapper + stealth config | Module gọi `core::crawler::playwright_run(script, params)` |
| **Proxy Pool** | Round-robin, health check, 2 pools | Module nhận proxy URL từ Core |
| **Scheduler** | Batch cron + Stream loop | Module đăng ký task vào scheduler |
| **Config** | ArcSwap hot-reload | Module đọc config qua `core::config::get()` |
| **Storage** | SQLite pool, migration runner | Module tạo DB riêng, dùng helper của Core |
| **API Gateway** | Axum router, CORS, auth | Module đăng ký routes vào router |
| **Registry** | DashMap + SQLite | Module tự đăng ký vào registry |
| **Common Types** | Score, Recommendation, Error | Module import types chung |

### Module tự làm GÌ riêng?

| Phần | stock_vn tự làm | msu_game tự làm |
|---|---|---|
| **Crawler logic** | Parse Simplize HTML, Yahoo JSON | Parse MSU marketplace API |
| **Valuation** | 20+ TA indicators, Fundamental, DCF | Rarity, Pricing, Deal Finder |
| **Trade** | Stub (chưa trade) | Playwright auto buy/sell |
| **Models** | StockData, OHLCV, FundamentalData | NftItem, Character, Deal |
| **DB Schema** | price_history, fundamental_data | nft_items, sales_history, deals |
| **Config riêng** | symbols watchlist, schedule | categories, deal thresholds |

---

## Dependency Graph

```
core ← stock_vn   (stock_vn phụ thuộc core)
core ← msu_game   (msu_game phụ thuộc core)

stock_vn ↔ msu_game  (KHÔNG phụ thuộc nhau)
```

### Cargo.toml (Workspace)
```toml
[workspace]
members = [
    "core",
    "modules/stock_vn",
    "modules/msu_game",
]
```

### modules/stock_vn/Cargo.toml
```toml
[dependencies]
core = { path = "../../core" }
# + domain-specific deps
```

### modules/msu_game/Cargo.toml
```toml
[dependencies]
core = { path = "../../core" }
# + domain-specific deps
```

---

## Thêm Module Mới (Tương Lai)

Khi cần thêm game/sàn mới (ví dụ: forex, pixels_game):

```
1. Tạo thư mục: modules/forex/ hoặc modules/pixels_game/
2. Copy template từ module có sẵn
3. Implement: crawler/, valuation/, trade/ riêng
4. Đăng ký vào Cargo.toml workspace
5. Đăng ký vào config/modules.toml
6. Core tự nhận diện module mới qua registry
```
