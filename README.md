# AI Config Pricing Engine

> Hệ thống định giá thông minh cho NFT & Cổ phiếu, viết bằng Rust.

## Tính năng

- **NFT Valuation**: Định giá NFT qua multiple marketplaces (OpenSea, Magic Eden, Blur)
- **Stock Technical Analysis**: SMA, EMA, RSI, Bollinger Bands, Volatility
- **Stock Fundamental Analysis**: P/E, P/B, EPS, cổ tức, DCF → khuyến nghị mua/bán
- **Dynamic Config**: Hot-reload cấu hình runtime via ArcSwap
- **Proxy Pool**: Multi-proxy rotation tránh rate-limiting
- **REST API**: 13 endpoints (Axum framework)
- **CrewAI Integration**: Webhook nhận dữ liệu từ AI spinner

## Quick Start

```bash
# Build
cargo build

# Run
cargo run

# Server sẽ chạy tại http://127.0.0.1:8080
```

## Cấu hình

- `config/default.toml` — Cấu hình mặc định
- `.env` — Environment variables (copy từ `.env.example`)

## API Endpoints

| Method | Path | Mô tả |
|--------|------|--------|
| `GET` | `/api/v1/health` | Health check |
| `POST` | `/api/v1/valuate/nft` | Định giá NFT |
| `POST` | `/api/v1/valuate/stock` | Phân tích technical |
| `POST` | `/api/v1/valuate/stock/fundamental` | Phân tích fundamental |
| `POST` | `/api/v1/valuate/batch` | Batch valuation |
| `GET/PUT` | `/api/v1/config` | Quản lý cấu hình |
| `POST` | `/api/v1/crew/webhook` | CrewAI webhook |

## Tech Stack

- **Rust** + Tokio async runtime
- **Axum** — REST API
- **SQLite** (sqlx) — Storage
- **ArcSwap** — Hot-reload config
- **reqwest** — HTTP client + proxy

## Roadmap

- [x] v0.1.0 — Core engine, NFT/Stock valuators, REST API, CrewAI integration
- [x] v0.2.0 — Fundamental Analysis, Proxy Pool, Recommendation engine
- [ ] v0.3.0 — Redis caching, WebSocket real-time updates
- [ ] v0.4.0 — HMAC-SHA256 webhook verification, API authentication
