// === Fundamental Analysis - Phân tích cơ bản cổ phiếu ===
// Đánh giá chất lượng dựa trên P/E, P/B, EPS, cổ tức, DCF
// Xác định giá trị nội tại và khuyến nghị mua/bán

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use super::Recommendation;

/// Dữ liệu tài chính đầu vào cho phân tích cơ bản
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalData {
    /// Giá cổ phiếu hiện tại
    pub current_price: f64,
    /// Thu nhập trên mỗi cổ phiếu (Earnings Per Share)
    pub eps: f64,
    /// Tỷ lệ giá/thu nhập (Price-to-Earnings)
    pub pe_ratio: f64,
    /// Tỷ lệ giá/giá trị sổ sách (Price-to-Book)
    pub pb_ratio: f64,
    /// Tỷ suất cổ tức (%)
    #[serde(default)]
    pub dividend_yield: f64,
    /// Tỷ lệ chi trả cổ tức (Payout Ratio)
    #[serde(default)]
    pub payout_ratio: Option<f64>,
    /// Tăng trưởng doanh thu (%)
    #[serde(default)]
    pub revenue_growth: f64,
    /// Tăng trưởng EPS (%) - so với cùng kỳ năm trước
    #[serde(default)]
    pub eps_growth: Option<f64>,
    /// Tỷ lệ nợ/vốn chủ sở hữu
    #[serde(default)]
    pub debt_to_equity: f64,
    /// Tỷ suất sinh lời trên vốn chủ sở hữu (Return on Equity)
    #[serde(default)]
    pub roe: f64,
    /// Dòng tiền tự do (Free Cash Flow)
    #[serde(default)]
    pub free_cash_flow: Option<f64>,
    /// Số cổ phiếu đang lưu hành
    #[serde(default)]
    pub shares_outstanding: Option<f64>,
    /// P/E trung bình ngành (để so sánh)
    #[serde(default)]
    pub industry_pe: Option<f64>,
    /// P/B trung bình ngành
    #[serde(default)]
    pub industry_pb: Option<f64>,
    /// Book value per share (giá trị sổ sách mỗi cổ phiếu)
    #[serde(default)]
    pub book_value_per_share: Option<f64>,
    /// Tỷ lệ tăng trưởng dự kiến (%) - cho DCF
    #[serde(default)]
    pub expected_growth_rate: Option<f64>,
}

/// Kết quả phân tích cơ bản
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalAnalysisResult {
    /// Mã cổ phiếu
    pub symbol: String,
    /// Giá hiện tại
    pub current_price: f64,
    /// Giá trị nội tại ước tính
    pub intrinsic_value: f64,
    /// Biên an toàn (%) - chênh lệch giữa intrinsic value và market price
    pub margin_of_safety: f64,
    /// Khuyến nghị tổng hợp
    pub recommendation: Recommendation,
    /// Điểm tổng hợp (0-100)
    pub composite_score: f64,
    /// Chi tiết phân tích từng chỉ số
    pub metric_scores: HashMap<String, MetricScore>,
    /// Điểm ổn định (0-100)
    pub stability_score: f64,
    /// Đánh giá định giá: overvalued, undervalued, fair
    pub valuation_status: ValuationStatus,
    /// Ghi chú phân tích
    pub analysis_notes: Vec<String>,
}

/// Đánh giá trạng thái định giá
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ValuationStatus {
    /// Định giá thấp - cơ hội mua
    Undervalued,
    /// Định giá hợp lý
    FairValued,
    /// Định giá cao - cân nhắc bán
    Overvalued,
    /// Định giá quá cao - nên bán
    SignificantlyOvervalued,
    /// Thiếu dữ liệu
    Undetermined,
}

/// Chi tiết điểm cho một chỉ số
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricScore {
    /// Giá trị thực tế
    pub value: f64,
    /// Điểm số (0-100)
    pub score: f64,
    /// Trọng số áp dụng
    pub weight: f64,
    /// Đóng góp vào composite score
    pub contribution: f64,
    /// Nhận xét
    pub comment: String,
}

/// Cấu hình cho fundamental analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalConfig {
    /// Tỷ lệ chiết khấu cho DCF (mặc định 10%)
    #[serde(default = "default_discount_rate")]
    pub discount_rate: f64,
    /// Tỷ lệ tăng trưởng terminal cho DCF (mặc định 3%)
    #[serde(default = "default_terminal_growth")]
    pub terminal_growth_rate: f64,
    /// Số năm dự phóng DCF (mặc định 10)
    #[serde(default = "default_projection_years")]
    pub projection_years: u32,
    /// Ngưỡng P/E coi là đắt (mặc định 25)
    #[serde(default = "default_pe_high")]
    pub pe_high_threshold: f64,
    /// Ngưỡng P/E coi là rẻ (mặc định 15)
    #[serde(default = "default_pe_low")]
    pub pe_low_threshold: f64,
    /// Ngưỡng margin of safety để khuyến nghị mua (mặc định 20%)
    #[serde(default = "default_mos_threshold")]
    pub margin_of_safety_threshold: f64,
}

fn default_discount_rate() -> f64 { 0.10 }
fn default_terminal_growth() -> f64 { 0.03 }
fn default_projection_years() -> u32 { 10 }
fn default_pe_high() -> f64 { 25.0 }
fn default_pe_low() -> f64 { 15.0 }
fn default_mos_threshold() -> f64 { 0.20 }

impl Default for FundamentalConfig {
    fn default() -> Self {
        Self {
            discount_rate: 0.10,
            terminal_growth_rate: 0.03,
            projection_years: 10,
            pe_high_threshold: 25.0,
            pe_low_threshold: 15.0,
            margin_of_safety_threshold: 0.20,
        }
    }
}

/// Fundamental Analyzer - phân tích cơ bản cổ phiếu
pub struct FundamentalAnalyzer {
    config: FundamentalConfig,
}

impl FundamentalAnalyzer {
    /// Khởi tạo analyzer với cấu hình
    pub fn new(config: FundamentalConfig) -> Self {
        Self { config }
    }

    /// Khởi tạo với cấu hình mặc định
    pub fn with_defaults() -> Self {
        Self::new(FundamentalConfig::default())
    }

    /// Phân tích toàn diện cổ phiếu
    pub fn analyze(&self, symbol: &str, data: &FundamentalData) -> FundamentalAnalysisResult {
        info!("Bắt đầu phân tích cơ bản cho: {}", symbol);

        let mut metric_scores = HashMap::new();
        let mut analysis_notes = Vec::new();

        // === 1. Phân tích P/E ===
        let pe_score = self.analyze_pe(data, &mut analysis_notes);
        metric_scores.insert("pe_ratio".to_string(), pe_score);

        // === 2. Phân tích P/B ===
        let pb_score = self.analyze_pb(data, &mut analysis_notes);
        metric_scores.insert("pb_ratio".to_string(), pb_score);

        // === 3. Phân tích EPS Growth ===
        let eps_score = self.analyze_eps_growth(data, &mut analysis_notes);
        metric_scores.insert("eps_growth".to_string(), eps_score);

        // === 4. Phân tích Cổ tức ===
        let div_score = self.analyze_dividend(data, &mut analysis_notes);
        metric_scores.insert("dividend".to_string(), div_score);

        // === 5. DCF Valuation ===
        let dcf_score = self.analyze_dcf(data, &mut analysis_notes);
        metric_scores.insert("dcf_margin".to_string(), dcf_score);

        // === 6. Stability Score ===
        let stability = self.calculate_stability(data, &mut analysis_notes);
        metric_scores.insert("stability".to_string(), stability);

        // === Tính composite score ===
        let mut composite = 0.0;
        let mut total_weight = 0.0;
        for score in metric_scores.values() {
            composite += score.score * score.weight;
            total_weight += score.weight;
        }
        if total_weight > 0.0 {
            composite /= total_weight;
        }

        // === Tính intrinsic value ===
        let intrinsic_value = self.calculate_intrinsic_value(data);

        // === Margin of Safety ===
        let margin_of_safety = if intrinsic_value > 0.0 {
            (intrinsic_value - data.current_price) / intrinsic_value
        } else {
            0.0
        };

        // === Xác định trạng thái định giá ===
        let valuation_status = self.determine_valuation_status(
            data.current_price, intrinsic_value, margin_of_safety,
        );

        // === Khuyến nghị ===
        let recommendation = self.generate_recommendation(
            composite, margin_of_safety, &valuation_status,
        );

        let stability_score = metric_scores.get("stability")
            .map(|s| s.score)
            .unwrap_or(50.0);

        info!(
            "Fundamental {}: score={:.1} | intrinsic={:.2} | MoS={:.1}% | {:?} | {:?}",
            symbol, composite, intrinsic_value,
            margin_of_safety * 100.0, valuation_status, recommendation
        );

        FundamentalAnalysisResult {
            symbol: symbol.to_string(),
            current_price: data.current_price,
            intrinsic_value,
            margin_of_safety,
            recommendation,
            composite_score: composite,
            metric_scores,
            stability_score,
            valuation_status,
            analysis_notes,
        }
    }

    // === Phân tích chi tiết từng chỉ số ===

    /// Phân tích P/E ratio
    fn analyze_pe(&self, data: &FundamentalData, notes: &mut Vec<String>) -> MetricScore {
        let pe = data.pe_ratio;
        let industry_pe = data.industry_pe.unwrap_or(20.0);

        // PE thấp = tốt (undervalued), PE cao = xấu (overvalued)
        // Nhưng PE quá thấp có thể là dấu hiệu xấu (earning issues)
        let score = if pe <= 0.0 {
            // PE âm = lỗ
            notes.push("⚠️ P/E âm: công ty đang thua lỗ".to_string());
            10.0
        } else if pe < 5.0 {
            // Quá rẻ hoặc có vấn đề
            notes.push("⚠️ P/E rất thấp (<5): có thể có rủi ro tiềm ẩn".to_string());
            50.0
        } else if pe < self.config.pe_low_threshold {
            // Rẻ - tốt
            notes.push(format!("✅ P/E={:.1}: cổ phiếu có vẻ được định giá thấp", pe));
            85.0
        } else if pe < self.config.pe_high_threshold {
            // Hợp lý
            notes.push(format!("ℹ️ P/E={:.1}: định giá hợp lý (ngành: {:.1})", pe, industry_pe));
            // Chuẩn hoá giữa low và high threshold
            let range = self.config.pe_high_threshold - self.config.pe_low_threshold;
            let position = (pe - self.config.pe_low_threshold) / range;
            70.0 - position * 20.0 // 70 → 50
        } else if pe < 40.0 {
            // Đắt
            notes.push(format!("⚠️ P/E={:.1}: cổ phiếu đang bị định giá cao", pe));
            30.0
        } else {
            // Rất đắt
            notes.push(format!("🔴 P/E={:.1}: cổ phiếu bị định giá quá cao", pe));
            10.0
        };

        // Điều chỉnh theo ngành
        let pe_vs_industry = pe / industry_pe;
        let adjusted_score = if pe_vs_industry < 0.8 {
            (score + 10.0).min(100.0) // Rẻ hơn ngành → cộng điểm
        } else if pe_vs_industry > 1.5 {
            (score - 15.0).max(0.0) // Đắt hơn ngành nhiều → trừ điểm
        } else {
            score
        };

        MetricScore {
            value: pe,
            score: adjusted_score,
            weight: 0.20,
            contribution: adjusted_score * 0.20,
            comment: format!(
                "P/E={:.1} vs ngành {:.1} (tỷ lệ: {:.2}x)",
                pe, industry_pe, pe_vs_industry
            ),
        }
    }

    /// Phân tích P/B ratio
    fn analyze_pb(&self, data: &FundamentalData, notes: &mut Vec<String>) -> MetricScore {
        let pb = data.pb_ratio;
        let industry_pb = data.industry_pb.unwrap_or(3.0);

        let score: f64 = if pb <= 0.0 {
            notes.push("⚠️ P/B âm: vốn chủ sở hữu âm".to_string());
            5.0
        } else if pb < 1.0 {
            // Giá thấp hơn giá trị sổ sách - rất hấp dẫn (hoặc có vấn đề)
            notes.push(format!("✅ P/B={:.2}: giá thấp hơn giá trị sổ sách", pb));
            90.0
        } else if pb < 2.0 {
            notes.push(format!("✅ P/B={:.2}: định giá hợp lý theo tài sản", pb));
            75.0
        } else if pb < 5.0 {
            notes.push(format!("ℹ️ P/B={:.2}: định giá cao so với tài sản", pb));
            50.0
        } else {
            notes.push(format!("⚠️ P/B={:.2}: định giá rất cao so với tài sản", pb));
            20.0
        };

        // So sánh với ngành
        let pb_vs_industry = pb / industry_pb;
        let adjusted_score = if pb_vs_industry < 0.7 {
            (score + 10.0).min(100.0)
        } else if pb_vs_industry > 2.0 {
            (score - 10.0).max(0.0)
        } else {
            score
        };

        MetricScore {
            value: pb,
            score: adjusted_score,
            weight: 0.15,
            contribution: adjusted_score * 0.15,
            comment: format!("P/B={:.2} vs ngành {:.2}", pb, industry_pb),
        }
    }

    /// Phân tích tăng trưởng EPS
    fn analyze_eps_growth(&self, data: &FundamentalData, notes: &mut Vec<String>) -> MetricScore {
        let eps = data.eps;
        let eps_growth = data.eps_growth.unwrap_or(0.0);

        let score = if eps <= 0.0 {
            notes.push("🔴 EPS âm: công ty đang lỗ".to_string());
            5.0
        } else if eps_growth > 0.25 {
            notes.push(format!("✅ EPS tăng trưởng mạnh: {:.1}%", eps_growth * 100.0));
            95.0
        } else if eps_growth > 0.10 {
            notes.push(format!("✅ EPS tăng trưởng tốt: {:.1}%", eps_growth * 100.0));
            80.0
        } else if eps_growth > 0.0 {
            notes.push(format!("ℹ️ EPS tăng trưởng nhẹ: {:.1}%", eps_growth * 100.0));
            60.0
        } else if eps_growth > -0.10 {
            notes.push(format!("⚠️ EPS giảm nhẹ: {:.1}%", eps_growth * 100.0));
            35.0
        } else {
            notes.push(format!("🔴 EPS giảm mạnh: {:.1}%", eps_growth * 100.0));
            10.0
        };

        MetricScore {
            value: eps_growth,
            score,
            weight: 0.20,
            contribution: score * 0.20,
            comment: format!("EPS={:.2}, tăng trưởng={:.1}%", eps, eps_growth * 100.0),
        }
    }

    /// Phân tích cổ tức
    fn analyze_dividend(&self, data: &FundamentalData, notes: &mut Vec<String>) -> MetricScore {
        let div_yield = data.dividend_yield;
        let payout_ratio = data.payout_ratio.unwrap_or(0.0);

        let score: f64 = if div_yield <= 0.0 {
            // Không trả cổ tức - không nhất thiết xấu (growth stocks)
            notes.push("ℹ️ Không trả cổ tức (có thể là growth stock)".to_string());
            40.0 // Trung lập
        } else if div_yield < 0.02 {
            notes.push(format!("ℹ️ Cổ tức thấp: {:.2}%", div_yield * 100.0));
            50.0
        } else if div_yield < 0.05 {
            notes.push(format!("✅ Cổ tức tốt: {:.2}%", div_yield * 100.0));
            75.0
        } else if div_yield < 0.08 {
            notes.push(format!("✅ Cổ tức cao: {:.2}%", div_yield * 100.0));
            85.0
        } else {
            // Quá cao có thể không bền vững
            notes.push(format!("⚠️ Cổ tức rất cao: {:.2}% - kiểm tra tính bền vững", div_yield * 100.0));
            60.0
        };

        // Điều chỉnh theo payout ratio
        let adjusted_score = if payout_ratio > 0.0 {
            if payout_ratio > 0.90 {
                notes.push(format!("⚠️ Payout ratio quá cao: {:.0}% - có thể không bền vững", payout_ratio * 100.0));
                (score - 15.0).max(0.0)
            } else if payout_ratio > 0.60 {
                score
            } else {
                // Payout ratio thấp = room to grow
                (score + 5.0).min(100.0)
            }
        } else {
            score
        };

        MetricScore {
            value: div_yield,
            score: adjusted_score,
            weight: 0.15,
            contribution: adjusted_score * 0.15,
            comment: format!(
                "Yield={:.2}%, Payout={:.0}%",
                div_yield * 100.0, payout_ratio * 100.0
            ),
        }
    }

    /// Phân tích DCF (Discounted Cash Flow) đơn giản hoá
    fn analyze_dcf(&self, data: &FundamentalData, notes: &mut Vec<String>) -> MetricScore {
        let intrinsic = self.calculate_intrinsic_value(data);

        if intrinsic <= 0.0 {
            notes.push("⚠️ Không thể tính DCF: thiếu dữ liệu FCF/EPS".to_string());
            return MetricScore {
                value: 0.0,
                score: 50.0, // Trung lập khi không đủ dữ liệu
                weight: 0.15,
                contribution: 50.0 * 0.15,
                comment: "DCF không khả dụng do thiếu dữ liệu".to_string(),
            };
        }

        let margin = (intrinsic - data.current_price) / intrinsic;

        let score = if margin > 0.40 {
            notes.push(format!("✅ Rất hấp dẫn: giá hiện tại thấp hơn {:.0}% so với giá trị nội tại", margin * 100.0));
            95.0
        } else if margin > self.config.margin_of_safety_threshold {
            notes.push(format!("✅ Biên an toàn tốt: {:.0}%", margin * 100.0));
            80.0
        } else if margin > 0.0 {
            notes.push(format!("ℹ️ Biên an toàn thấp: {:.0}%", margin * 100.0));
            60.0
        } else if margin > -0.20 {
            notes.push(format!("⚠️ Đắt hơn giá trị nội tại: {:.0}%", margin.abs() * 100.0));
            35.0
        } else {
            notes.push(format!("🔴 Quá đắt so với giá trị nội tại: +{:.0}%", margin.abs() * 100.0));
            10.0
        };

        MetricScore {
            value: margin,
            score,
            weight: 0.15,
            contribution: score * 0.15,
            comment: format!(
                "Intrinsic={:.2}, Market={:.2}, MoS={:.1}%",
                intrinsic, data.current_price, margin * 100.0
            ),
        }
    }

    /// Tính điểm ổn định
    fn calculate_stability(&self, data: &FundamentalData, notes: &mut Vec<String>) -> MetricScore {
        let mut stability: f64 = 50.0; // Điểm baseline

        // Debt/Equity
        if data.debt_to_equity < 0.5 {
            stability += 20.0;
            notes.push(format!("✅ D/E thấp ({:.2}): tài chính lành mạnh", data.debt_to_equity));
        } else if data.debt_to_equity < 1.0 {
            stability += 10.0;
        } else if data.debt_to_equity < 2.0 {
            stability -= 5.0;
            notes.push(format!("⚠️ D/E khá cao ({:.2}): nợ nhiều", data.debt_to_equity));
        } else {
            stability -= 20.0;
            notes.push(format!("🔴 D/E rất cao ({:.2}): rủi ro tài chính", data.debt_to_equity));
        }

        // ROE
        if data.roe > 0.20 {
            stability += 15.0;
            notes.push(format!("✅ ROE cao ({:.1}%): hiệu quả sử dụng vốn tốt", data.roe * 100.0));
        } else if data.roe > 0.10 {
            stability += 5.0;
        } else if data.roe > 0.0 {
            stability -= 5.0;
        } else {
            stability -= 15.0;
            notes.push(format!("🔴 ROE âm ({:.1}%): công ty lỗ vốn", data.roe * 100.0));
        }

        // Revenue growth
        if data.revenue_growth > 0.15 {
            stability += 10.0;
        } else if data.revenue_growth > 0.05 {
            stability += 5.0;
        } else if data.revenue_growth < 0.0 {
            stability -= 10.0;
            notes.push(format!("⚠️ Doanh thu giảm: {:.1}%", data.revenue_growth * 100.0));
        }

        let final_score = stability.clamp(0.0, 100.0);

        MetricScore {
            value: final_score,
            score: final_score,
            weight: 0.15,
            contribution: final_score * 0.15,
            comment: format!(
                "D/E={:.2}, ROE={:.1}%, Revenue Growth={:.1}%",
                data.debt_to_equity, data.roe * 100.0, data.revenue_growth * 100.0
            ),
        }
    }

    /// Tính giá trị nội tại bằng phương pháp kết hợp
    /// Kết hợp: DCF (nếu có FCF), Graham formula, Net Asset Value
    fn calculate_intrinsic_value(&self, data: &FundamentalData) -> f64 {
        let mut values = Vec::new();
        let mut weights = Vec::new();

        // === 1. DCF Valuation (trọng số cao nhất) ===
        if let Some(fcf) = data.free_cash_flow {
            if fcf > 0.0 {
                if let Some(shares) = data.shares_outstanding {
                    if shares > 0.0 {
                        let dcf_value = self.calculate_dcf(
                            fcf,
                            shares,
                            data.expected_growth_rate.unwrap_or(data.revenue_growth.max(0.02)),
                        );
                        if dcf_value > 0.0 {
                            values.push(dcf_value);
                            weights.push(0.50); // Trọng số 50%
                        }
                    }
                }
            }
        }

        // === 2. Graham Number (công thức Benjamin Graham) ===
        // Intrinsic Value = sqrt(22.5 × EPS × Book Value)
        if data.eps > 0.0 {
            if let Some(bvps) = data.book_value_per_share {
                if bvps > 0.0 {
                    let graham = (22.5 * data.eps * bvps).sqrt();
                    values.push(graham);
                    weights.push(0.30);
                }
            } else if data.pb_ratio > 0.0 {
                // Tính book value từ P/B
                let bvps = data.current_price / data.pb_ratio;
                let graham = (22.5 * data.eps * bvps).sqrt();
                values.push(graham);
                weights.push(0.30);
            }
        }

        // === 3. Earnings-based (P/E fair value) ===
        if data.eps > 0.0 {
            let fair_pe = data.industry_pe.unwrap_or(18.0);
            let earnings_value = data.eps * fair_pe;
            values.push(earnings_value);
            weights.push(0.20);
        }

        // Tính weighted average
        if values.is_empty() {
            return 0.0;
        }

        let total_weight: f64 = weights.iter().sum();
        let intrinsic: f64 = values.iter().zip(weights.iter())
            .map(|(v, w)| v * w)
            .sum::<f64>() / total_weight;

        intrinsic
    }

    /// Tính DCF value per share
    fn calculate_dcf(
        &self,
        free_cash_flow: f64,
        shares_outstanding: f64,
        growth_rate: f64,
    ) -> f64 {
        let r = self.config.discount_rate;
        let g = growth_rate.min(r - 0.01); // Growth phải nhỏ hơn discount rate
        let terminal_g = self.config.terminal_growth_rate;
        let n = self.config.projection_years;

        // Phase 1: Dự phóng FCF cho n năm
        let mut total_pv = 0.0;
        let mut projected_fcf = free_cash_flow;

        for year in 1..=n {
            projected_fcf *= 1.0 + g;
            let discount_factor = (1.0 + r).powi(year as i32);
            total_pv += projected_fcf / discount_factor;
        }

        // Phase 2: Terminal Value (Gordon Growth Model)
        let terminal_fcf = projected_fcf * (1.0 + terminal_g);
        let terminal_value = if r > terminal_g {
            terminal_fcf / (r - terminal_g)
        } else {
            0.0
        };
        let terminal_pv = terminal_value / (1.0 + r).powi(n as i32);

        // Tổng giá trị / số cổ phiếu
        let total_value = total_pv + terminal_pv;
        total_value / shares_outstanding
    }

    /// Xác định trạng thái định giá
    fn determine_valuation_status(
        &self,
        _market_price: f64,
        intrinsic_value: f64,
        margin_of_safety: f64,
    ) -> ValuationStatus {
        if intrinsic_value <= 0.0 {
            return ValuationStatus::Undetermined;
        }

        if margin_of_safety > self.config.margin_of_safety_threshold {
            ValuationStatus::Undervalued
        } else if margin_of_safety > -0.10 {
            ValuationStatus::FairValued
        } else if margin_of_safety > -0.30 {
            ValuationStatus::Overvalued
        } else {
            ValuationStatus::SignificantlyOvervalued
        }
    }

    /// Tổng hợp khuyến nghị từ tất cả chỉ số
    fn generate_recommendation(
        &self,
        composite_score: f64,
        margin_of_safety: f64,
        valuation: &ValuationStatus,
    ) -> Recommendation {
        // Kết hợp composite score + margin of safety + valuation status
        let mut final_score = composite_score;

        // Điều chỉnh theo margin of safety
        if margin_of_safety > 0.30 {
            final_score += 15.0;
        } else if margin_of_safety > 0.15 {
            final_score += 8.0;
        } else if margin_of_safety < -0.20 {
            final_score -= 15.0;
        } else if margin_of_safety < -0.10 {
            final_score -= 8.0;
        }

        // Điều chỉnh theo valuation
        match valuation {
            ValuationStatus::Undervalued => final_score += 5.0,
            ValuationStatus::SignificantlyOvervalued => final_score -= 10.0,
            ValuationStatus::Overvalued => final_score -= 5.0,
            _ => {}
        }

        let final_score = final_score.clamp(0.0, 100.0);

        match final_score {
            s if s >= 80.0 => Recommendation::StrongBuy,
            s if s >= 65.0 => Recommendation::Buy,
            s if s >= 40.0 => Recommendation::Hold,
            s if s >= 25.0 => Recommendation::Sell,
            _ => Recommendation::StrongSell,
        }
    }
}

/// Parse FundamentalData từ JSON (hỗ trợ nhiều format tên field)
pub fn parse_fundamental_from_json(data: &serde_json::Value) -> anyhow::Result<FundamentalData> {
    let extract = |names: &[&str]| -> Option<f64> {
        for name in names {
            if let Some(v) = data.get(*name) {
                if let Some(n) = v.as_f64() { return Some(n); }
                if let Some(s) = v.as_str() {
                    if let Ok(n) = s.parse::<f64>() { return Some(n); }
                }
            }
        }
        None
    };

    Ok(FundamentalData {
        current_price: extract(&["current_price", "price", "currentPrice"])
            .ok_or_else(|| anyhow::anyhow!("Thiếu current_price"))?,
        eps: extract(&["eps", "EPS", "earnings_per_share"])
            .ok_or_else(|| anyhow::anyhow!("Thiếu EPS"))?,
        pe_ratio: extract(&["pe_ratio", "PE", "pe", "priceToEarnings"])
            .ok_or_else(|| anyhow::anyhow!("Thiếu P/E ratio"))?,
        pb_ratio: extract(&["pb_ratio", "PB", "pb", "priceToBook"])
            .unwrap_or(0.0),
        dividend_yield: extract(&["dividend_yield", "dividendYield", "div_yield"])
            .unwrap_or(0.0),
        payout_ratio: extract(&["payout_ratio", "payoutRatio"]),
        revenue_growth: extract(&["revenue_growth", "revenueGrowth"])
            .unwrap_or(0.0),
        eps_growth: extract(&["eps_growth", "epsGrowth"]),
        debt_to_equity: extract(&["debt_to_equity", "debtToEquity", "DE"])
            .unwrap_or(0.0),
        roe: extract(&["roe", "ROE", "returnOnEquity"])
            .unwrap_or(0.0),
        free_cash_flow: extract(&["free_cash_flow", "freeCashFlow", "FCF"]),
        shares_outstanding: extract(&["shares_outstanding", "sharesOutstanding"]),
        industry_pe: extract(&["industry_pe", "industryPE"]),
        industry_pb: extract(&["industry_pb", "industryPB"]),
        book_value_per_share: extract(&["book_value_per_share", "bookValuePerShare", "BVPS"]),
        expected_growth_rate: extract(&["expected_growth_rate", "expectedGrowth"]),
    })
}
