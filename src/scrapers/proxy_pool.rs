// === Proxy Pool - Quản lý proxy đa luồng cho scraping ===
// Round-robin rotation, health check, failover tự động
// Tránh rate limiting bằng cách phân tải request qua nhiều proxy

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use tokio::sync::RwLock;

/// Loại proxy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Http,
    Https,
    Socks5,
}

impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::Http
    }
}

/// Cấu hình cho một proxy server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Địa chỉ proxy (IP hoặc hostname)
    pub address: String,
    /// Cổng
    pub port: u16,
    /// Loại proxy
    #[serde(default)]
    pub proxy_type: ProxyType,
    /// Username (tuỳ chọn)
    #[serde(default)]
    pub username: Option<String>,
    /// Password (tuỳ chọn)
    #[serde(default)]
    pub password: Option<String>,
    /// Tên hiển thị (tuỳ chọn)
    #[serde(default)]
    pub label: Option<String>,
}

impl ProxyConfig {
    /// Tạo URL proxy đầy đủ
    pub fn to_url(&self) -> String {
        let scheme = match self.proxy_type {
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks5 => "socks5",
        };

        match (&self.username, &self.password) {
            (Some(user), Some(pass)) => {
                format!("{}://{}:{}@{}:{}", scheme, user, pass, self.address, self.port)
            }
            _ => {
                format!("{}://{}:{}", scheme, self.address, self.port)
            }
        }
    }
}

/// Trạng thái runtime của một proxy
struct ProxyState {
    /// Cấu hình proxy
    config: ProxyConfig,
    /// Proxy có khả dụng không
    is_alive: AtomicBool,
    /// Số request thành công
    success_count: AtomicUsize,
    /// Số request thất bại
    fail_count: AtomicUsize,
}

impl ProxyState {
    fn new(config: ProxyConfig) -> Self {
        Self {
            config,
            is_alive: AtomicBool::new(true),
            success_count: AtomicUsize::new(0),
            fail_count: AtomicUsize::new(0),
        }
    }

    /// Tỷ lệ thành công
    fn success_rate(&self) -> f64 {
        let success = self.success_count.load(Ordering::Relaxed) as f64;
        let fail = self.fail_count.load(Ordering::Relaxed) as f64;
        let total = success + fail;
        if total == 0.0 { 1.0 } else { success / total }
    }

    /// Đánh dấu request thành công
    fn mark_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        // Tự động phục hồi proxy nếu thành công lại
        self.is_alive.store(true, Ordering::Relaxed);
    }

    /// Đánh dấu request thất bại
    fn mark_failure(&self) {
        self.fail_count.fetch_add(1, Ordering::Relaxed);
        // Nếu tỷ lệ thất bại quá cao → đánh dấu dead
        if self.success_rate() < 0.3 && self.fail_count.load(Ordering::Relaxed) > 5 {
            warn!(
                "Proxy {} bị đánh dấu dead (success rate: {:.0}%)",
                self.config.to_url(), self.success_rate() * 100.0
            );
            self.is_alive.store(false, Ordering::Relaxed);
        }
    }
}

/// Cấu hình tổng hợp cho proxy pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPoolConfig {
    /// Bật/tắt proxy pool
    #[serde(default)]
    pub enabled: bool,
    /// Danh sách proxy servers
    #[serde(default)]
    pub proxies: Vec<ProxyConfig>,
    /// Thời gian giữa mỗi lần health check (giây)
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval_secs: u64,
    /// Tự động retry khi proxy thất bại
    #[serde(default = "default_auto_retry")]
    pub auto_retry_on_failure: bool,
    /// Số proxy tối đa sử dụng đồng thời
    #[serde(default)]
    pub max_concurrent: Option<usize>,
}

fn default_health_check_interval() -> u64 { 60 }
fn default_auto_retry() -> bool { true }

impl Default for ProxyPoolConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            proxies: Vec::new(),
            health_check_interval_secs: 60,
            auto_retry_on_failure: true,
            max_concurrent: None,
        }
    }
}

/// Proxy Pool Manager - quản lý danh sách proxy và round-robin rotation
pub struct ProxyPool {
    /// Danh sách proxy states
    proxies: Vec<Arc<ProxyState>>,
    /// Chỉ số proxy tiếp theo (round-robin)
    next_index: AtomicUsize,
    /// Cấu hình pool
    config: ProxyPoolConfig,
}

impl ProxyPool {
    /// Khởi tạo proxy pool từ config
    pub fn new(config: ProxyPoolConfig) -> Self {
        let proxies: Vec<Arc<ProxyState>> = config.proxies.iter()
            .map(|pc| Arc::new(ProxyState::new(pc.clone())))
            .collect();

        info!("Khởi tạo ProxyPool: {} proxies", proxies.len());

        Self {
            proxies,
            next_index: AtomicUsize::new(0),
            config,
        }
    }

    /// Kiểm tra pool có proxy nào không
    pub fn is_empty(&self) -> bool {
        self.proxies.is_empty()
    }

    /// Số proxy đang hoạt động
    pub fn alive_count(&self) -> usize {
        self.proxies.iter()
            .filter(|p| p.is_alive.load(Ordering::Relaxed))
            .count()
    }

    /// Lấy proxy tiếp theo (round-robin, bỏ qua dead proxies)
    pub fn get_next_proxy(&self) -> Option<ProxyConfig> {
        if self.proxies.is_empty() {
            return None;
        }

        let total = self.proxies.len();
        let start = self.next_index.fetch_add(1, Ordering::Relaxed) % total;

        // Tìm proxy alive tiếp theo (round-robin)
        for i in 0..total {
            let idx = (start + i) % total;
            let proxy = &self.proxies[idx];
            if proxy.is_alive.load(Ordering::Relaxed) {
                return Some(proxy.config.clone());
            }
        }

        // Nếu tất cả dead, reset và dùng proxy đầu tiên
        warn!("Tất cả proxy đều dead! Reset và dùng proxy đầu tiên.");
        for proxy in &self.proxies {
            proxy.is_alive.store(true, Ordering::Relaxed);
        }
        self.proxies.first().map(|p| p.config.clone())
    }

    /// Báo cáo request thành công cho proxy
    pub fn report_success(&self, proxy_url: &str) {
        if let Some(state) = self.find_proxy_state(proxy_url) {
            state.mark_success();
        }
    }

    /// Báo cáo request thất bại cho proxy
    pub fn report_failure(&self, proxy_url: &str) {
        if let Some(state) = self.find_proxy_state(proxy_url) {
            state.mark_failure();
        }
    }

    /// Tìm proxy state theo URL
    fn find_proxy_state(&self, proxy_url: &str) -> Option<&Arc<ProxyState>> {
        self.proxies.iter().find(|p| p.config.to_url() == proxy_url)
    }

    /// Lấy thống kê proxy pool
    pub fn get_stats(&self) -> Vec<ProxyStats> {
        self.proxies.iter().map(|p| ProxyStats {
            address: format!("{}:{}", p.config.address, p.config.port),
            label: p.config.label.clone(),
            is_alive: p.is_alive.load(Ordering::Relaxed),
            success_count: p.success_count.load(Ordering::Relaxed),
            fail_count: p.fail_count.load(Ordering::Relaxed),
            success_rate: p.success_rate(),
        }).collect()
    }

    /// Tạo reqwest::Client với proxy cụ thể
    pub fn create_proxied_client(
        proxy_config: &ProxyConfig,
        timeout_secs: u64,
        user_agent: &str,
    ) -> anyhow::Result<reqwest::Client> {
        let proxy_url = proxy_config.to_url();
        let proxy = reqwest::Proxy::all(&proxy_url)?;

        let client = reqwest::Client::builder()
            .proxy(proxy)
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(user_agent)
            .build()?;

        Ok(client)
    }
}

/// Thống kê cho một proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStats {
    /// Địa chỉ proxy
    pub address: String,
    /// Tên hiển thị
    pub label: Option<String>,
    /// Trạng thái hoạt động
    pub is_alive: bool,
    /// Số request thành công
    pub success_count: usize,
    /// Số request thất bại
    pub fail_count: usize,
    /// Tỷ lệ thành công
    pub success_rate: f64,
}
