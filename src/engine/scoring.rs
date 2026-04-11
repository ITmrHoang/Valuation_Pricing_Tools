// === AI Scoring Engine - Hệ thống chấm điểm thông minh ===
// Weighted scoring, normalization, cross-marketplace comparison

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::DynamicConfigManager;
use crate::config::attributes::WeightConfig;

/// Kết quả scoring chi tiết
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Điểm tổng hợp (0-100)
    pub total_score: f64,
    /// Điểm từng thuộc tính (đã chuẩn hoá 0-100)
    pub attribute_scores: HashMap<String, f64>,
    /// Trọng số đã áp dụng
    pub applied_weights: HashMap<String, f64>,
    /// Đóng góp của từng thuộc tính vào tổng điểm
    pub contributions: HashMap<String, f64>,
}

/// Scoring Engine - tính điểm dựa trên dynamic attributes + weights
pub struct ScoringEngine {
    config: Arc<DynamicConfigManager>,
}

impl ScoringEngine {
    /// Khởi tạo scoring engine
    pub fn new(config: Arc<DynamicConfigManager>) -> Self {
        Self { config }
    }

    /// Tính điểm tổng hợp từ raw scores + weights
    pub fn calculate_composite_score(
        &self,
        raw_scores: &HashMap<String, f64>,
        weights: &WeightConfig,
    ) -> ScoreBreakdown {
        let mut normalized_scores = HashMap::new();
        let mut contributions = HashMap::new();
        let mut applied_weights = HashMap::new();
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        for (key, &raw_score) in raw_scores {
            // Chuẩn hoá score về 0-100
            let normalized = raw_score.clamp(0.0, 100.0);
            normalized_scores.insert(key.clone(), normalized);

            // Lấy trọng số
            let weight = weights.get_weight(key);
            applied_weights.insert(key.clone(), weight);

            // Tính đóng góp
            let contribution = normalized * weight;
            contributions.insert(key.clone(), contribution);

            total_score += contribution;
            total_weight += weight;
        }

        // Chuẩn hoá tổng nếu trọng số không bằng 1
        if total_weight > 0.0 && (total_weight - 1.0).abs() > 0.01 {
            total_score /= total_weight;
        }

        ScoreBreakdown {
            total_score,
            attribute_scores: normalized_scores,
            applied_weights,
            contributions,
        }
    }

    /// So sánh điểm giữa nhiều tài sản (cross-asset comparison)
    pub fn rank_assets(&self, scores: &[(String, f64)]) -> Vec<(String, f64, usize)> {
        let mut ranked: Vec<(String, f64, usize)> = scores.iter()
            .map(|(name, score)| (name.clone(), *score, 0))
            .collect();

        // Sắp xếp giảm dần theo score
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Gán thứ hạng
        for (i, item) in ranked.iter_mut().enumerate() {
            item.2 = i + 1;
        }

        ranked
    }

    /// Chuẩn hoá giá trị theo min-max scaling
    pub fn min_max_normalize(values: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }

        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if (max - min).abs() < f64::EPSILON {
            return vec![0.5; values.len()]; // Tất cả giống nhau
        }

        values.iter().map(|v| (v - min) / (max - min)).collect()
    }

    /// Chuẩn hoá giá trị theo Z-score
    pub fn z_score_normalize(values: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev < f64::EPSILON {
            return vec![0.0; values.len()];
        }

        values.iter().map(|v| (v - mean) / std_dev).collect()
    }

    /// Tính confidence level dựa trên số lượng dữ liệu có sẵn
    pub fn calculate_data_confidence(
        available_fields: usize,
        total_fields: usize,
        data_points: usize,
    ) -> f64 {
        if total_fields == 0 {
            return 0.0;
        }

        // Trọng số: 60% coverage, 40% data points
        let coverage = available_fields as f64 / total_fields as f64;
        let depth = match data_points {
            0 => 0.0,
            1..=10 => 0.3,
            11..=50 => 0.6,
            51..=200 => 0.85,
            _ => 1.0,
        };

        (coverage * 0.6 + depth * 0.4).min(1.0)
    }
}
