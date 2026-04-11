// === API Handlers - Xử lý request cho các endpoints ===

use std::sync::Arc;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::{info, error};

use super::AppState;
use super::models::*;
use crate::config::DynamicConfigManager;
use crate::config::attributes::{AttributeSet, WeightConfig};
use crate::engine::{AssetType, PricingEngine, ValuationRequest};
use crate::engine::nft_valuator::NftValuator;
use crate::engine::stock_valuator::StockValuator;
use crate::crew_integration::{CrewSpinnerData, CrewDataProcessor};
use crate::storage::models::ValuationRepository;
use crate::engine::fundamental_analysis::{
    FundamentalAnalyzer, FundamentalConfig, parse_fundamental_from_json,
};

/// GET /api/v1/health - Kiểm tra sức khoẻ hệ thống
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "valuation_pricing_tools",
        "version": "0.1.0"
    }))
}

/// GET /api/v1/config - Lấy cấu hình hiện tại
pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let config = state.config_manager.get_config();
    Json(serde_json::json!({
        "status": "success",
        "data": *config
    }))
}

/// PUT /api/v1/config - Cập nhật cấu hình
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("Nhận yêu cầu cập nhật config");

    // Parse config mới
    match serde_json::from_value::<crate::config::AppConfig>(payload) {
        Ok(new_config) => {
            state.config_manager.update_config(new_config);

            // Lưu snapshot
            let config_json = serde_json::to_string(&*state.config_manager.get_config())
                .unwrap_or_default();
            let _ = ValuationRepository::save_config_snapshot(
                &state.db_pool,
                &config_json,
                "API update",
            ).await;

            (StatusCode::OK, Json(serde_json::json!({
                "status": "success",
                "message": "Cấu hình đã được cập nhật"
            })))
        }
        Err(e) => {
            error!("Lỗi parse config: {}", e);
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "status": "error",
                "message": format!("Config không hợp lệ: {}", e)
            })))
        }
    }
}

/// GET /api/v1/config/attributes - Lấy dynamic attributes
pub async fn get_attributes(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let attrs = state.config_manager.get_attributes();
    Json(serde_json::json!({
        "status": "success",
        "data": *attrs
    }))
}

/// PUT /api/v1/config/attributes - Cập nhật dynamic attributes
pub async fn update_attributes(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AttributeSet>,
) -> impl IntoResponse {
    info!("Cập nhật dynamic attributes: {}", payload.name);
    state.config_manager.update_attributes(payload);
    
    (StatusCode::OK, Json(serde_json::json!({
        "status": "success",
        "message": "Attributes đã được cập nhật"
    })))
}

/// PUT /api/v1/config/weights/nft - Cập nhật trọng số NFT
pub async fn update_nft_weights(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WeightConfig>,
) -> impl IntoResponse {
    info!("Cập nhật trọng số NFT");
    state.config_manager.update_nft_weights(payload);
    
    (StatusCode::OK, Json(serde_json::json!({
        "status": "success",
        "message": "Trọng số NFT đã được cập nhật"
    })))
}

/// PUT /api/v1/config/weights/stock - Cập nhật trọng số Stock
pub async fn update_stock_weights(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WeightConfig>,
) -> impl IntoResponse {
    info!("Cập nhật trọng số Stock");
    state.config_manager.update_stock_weights(payload);
    
    (StatusCode::OK, Json(serde_json::json!({
        "status": "success",
        "message": "Trọng số Stock đã được cập nhật"
    })))
}

/// POST /api/v1/valuate/nft - Định giá NFT
pub async fn valuate_nft(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ValuateNftRequest>,
) -> impl IntoResponse {
    info!("Yêu cầu định giá NFT: {}", payload.identifier);

    let valuator = NftValuator::new(state.config_manager.clone());
    let request = ValuationRequest {
        asset_type: AssetType::Nft,
        identifier: payload.identifier,
        marketplace: payload.marketplace,
        additional_data: payload.data,
        weight_overrides: payload.weight_overrides,
    };

    match valuator.valuate(&request).await {
        Ok(result) => {
            // Lưu kết quả vào DB
            let _ = ValuationRepository::save(&state.db_pool, &result).await;
            
            (StatusCode::OK, Json(serde_json::json!({
                "status": "success",
                "data": result
            })))
        }
        Err(e) => {
            error!("Lỗi định giá NFT: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "status": "error",
                "message": format!("Lỗi định giá: {}", e)
            })))
        }
    }
}

/// POST /api/v1/valuate/stock - Phân tích cổ phiếu
pub async fn valuate_stock(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ValuateStockRequest>,
) -> impl IntoResponse {
    info!("Yêu cầu phân tích cổ phiếu: {}", payload.symbol);

    let valuator = StockValuator::new(state.config_manager.clone());
    let request = ValuationRequest {
        asset_type: AssetType::Stock,
        identifier: payload.symbol,
        marketplace: None,
        additional_data: payload.data,
        weight_overrides: payload.weight_overrides,
    };

    match valuator.valuate(&request).await {
        Ok(result) => {
            let _ = ValuationRepository::save(&state.db_pool, &result).await;
            
            (StatusCode::OK, Json(serde_json::json!({
                "status": "success",
                "data": result
            })))
        }
        Err(e) => {
            error!("Lỗi phân tích cổ phiếu: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "status": "error",
                "message": format!("Lỗi phân tích: {}", e)
            })))
        }
    }
}

/// POST /api/v1/valuate/batch - Định giá hàng loạt
pub async fn valuate_batch(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BatchValuationRequest>,
) -> impl IntoResponse {
    info!("Yêu cầu batch valuation: {} items", payload.requests.len());

    let mut all_results = Vec::new();

    // Tách NFT và Stock requests
    let nft_requests: Vec<_> = payload.requests.iter()
        .filter(|r| r.asset_type == "nft")
        .cloned()
        .collect();
    let stock_requests: Vec<_> = payload.requests.iter()
        .filter(|r| r.asset_type == "stock")
        .cloned()
        .collect();

    // Xử lý song song NFT và Stock
    let config = state.config_manager.clone();
    let db = state.db_pool.clone();

    // NFT batch
    if !nft_requests.is_empty() {
        let valuator = NftValuator::new(config.clone());
        let requests: Vec<ValuationRequest> = nft_requests.into_iter()
            .map(|r| ValuationRequest {
                asset_type: AssetType::Nft,
                identifier: r.identifier,
                marketplace: r.marketplace,
                additional_data: r.data,
                weight_overrides: None,
            })
            .collect();
        let results = valuator.valuate_batch(&requests).await;
        for result in results {
            if let Ok(r) = result {
                let _ = ValuationRepository::save(&db, &r).await;
                all_results.push(serde_json::to_value(&r).unwrap_or_default());
            }
        }
    }

    // Stock batch
    if !stock_requests.is_empty() {
        let valuator = StockValuator::new(config.clone());
        let requests: Vec<ValuationRequest> = stock_requests.into_iter()
            .map(|r| ValuationRequest {
                asset_type: AssetType::Stock,
                identifier: r.identifier,
                marketplace: None,
                additional_data: r.data,
                weight_overrides: None,
            })
            .collect();
        let results = valuator.valuate_batch(&requests).await;
        for result in results {
            if let Ok(r) = result {
                let _ = ValuationRepository::save(&db, &r).await;
                all_results.push(serde_json::to_value(&r).unwrap_or_default());
            }
        }
    }

    (StatusCode::OK, Json(serde_json::json!({
        "status": "success",
        "count": all_results.len(),
        "data": all_results
    })))
}

/// GET /api/v1/history/:asset_id - Lấy lịch sử định giá
pub async fn get_history(
    State(state): State<Arc<AppState>>,
    Path(asset_id): Path<String>,
) -> impl IntoResponse {
    info!("Lấy lịch sử định giá: {}", asset_id);

    match ValuationRepository::get_history(&state.db_pool, &asset_id, 50).await {
        Ok(history) => {
            (StatusCode::OK, Json(serde_json::json!({
                "status": "success",
                "asset": asset_id,
                "count": history.len(),
                "data": history
            })))
        }
        Err(e) => {
            error!("Lỗi lấy lịch sử: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "status": "error",
                "message": format!("Lỗi: {}", e)
            })))
        }
    }
}

/// POST /api/v1/crew/webhook - Nhận dữ liệu từ CrewAI Spinner
pub async fn crew_webhook(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CrewSpinnerData>,
) -> impl IntoResponse {
    info!("Nhận webhook CrewAI: task={}, type={}", payload.task_id, payload.data_type);

    // Xử lý dữ liệu
    let processed = CrewDataProcessor::process(&payload);

    if !processed.is_valid {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "status": "error",
            "message": "Dữ liệu không hợp lệ",
            "warnings": processed.warnings
        })));
    }

    // Tự động định giá nếu có đủ dữ liệu
    let valuation_result = match processed.asset_type.as_str() {
        "nft" => {
            let valuator = NftValuator::new(state.config_manager.clone());
            let request = ValuationRequest {
                asset_type: AssetType::Nft,
                identifier: processed.identifier.clone(),
                marketplace: Some(processed.source.clone()),
                additional_data: Some(processed.normalized_data.clone()),
                weight_overrides: None,
            };
            valuator.valuate(&request).await.ok()
        }
        "stock" => {
            let valuator = StockValuator::new(state.config_manager.clone());
            let request = ValuationRequest {
                asset_type: AssetType::Stock,
                identifier: processed.identifier.clone(),
                marketplace: None,
                additional_data: Some(processed.normalized_data.clone()),
                weight_overrides: None,
            };
            valuator.valuate(&request).await.ok()
        }
        _ => None,
    };

    // Lưu kết quả
    if let Some(result) = &valuation_result {
        let _ = ValuationRepository::save(&state.db_pool, result).await;
    }

    (StatusCode::OK, Json(serde_json::json!({
        "status": "success",
        "processed": processed,
        "valuation": valuation_result
    })))
}

/// POST /api/v1/valuate/stock/fundamental - Phân tích fundamental cổ phiếu
pub async fn valuate_stock_fundamental(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ValuateFundamentalRequest>,
) -> impl IntoResponse {
    info!("Yêu cầu phân tích fundamental: {}", payload.symbol);

    // Parse dữ liệu fundamental
    match parse_fundamental_from_json(&payload.fundamental_data) {
        Ok(fund_data) => {
            // Lấy config fundamental
            let config = state.config_manager.get_config();
            let fund_config = config.fundamental.clone();
            let analyzer = FundamentalAnalyzer::new(fund_config);

            // Nếu có bars → kết hợp technical + fundamental
            if let Some(bars_json) = &payload.bars {
                match crate::engine::stock_valuator::parse_ohlcv_from_json(bars_json) {
                    Ok(bars) if !bars.is_empty() => {
                        let valuator = StockValuator::new(state.config_manager.clone());
                        let result = valuator.valuate_with_fundamentals(
                            &payload.symbol, &bars, &fund_data,
                        );
                        let _ = ValuationRepository::save(&state.db_pool, &result).await;

                        return (StatusCode::OK, Json(serde_json::json!({
                            "status": "success",
                            "analysis_type": "combined_technical_fundamental",
                            "data": result
                        })));
                    }
                    _ => {
                        info!("Bars không hợp lệ, chỉ chạy fundamental analysis");
                    }
                }
            }

            // Chỉ fundamental analysis
            let fund_result = analyzer.analyze(&payload.symbol, &fund_data);

            (StatusCode::OK, Json(serde_json::json!({
                "status": "success",
                "analysis_type": "fundamental_only",
                "data": fund_result
            })))
        }
        Err(e) => {
            error!("Lỗi parse fundamental data: {}", e);
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "status": "error",
                "message": format!("Dữ liệu fundamental không hợp lệ: {}", e)
            })))
        }
    }
}
