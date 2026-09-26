use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
use tracing::{info, warn};

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init() {
    let path = match exe_config_path() {
        Some(p) => p,
        None => return,
    };
    if !path.exists() {
        // config.toml 不存在时生成默认配置；max_concurrent 填入实际 CPU 核心数作为基数
        let default_config = format!(
            "[server]\nport = 7878\nmax_concurrent = {}\ntimeout = 60\nstatus_route = false\n\n[routes]\ndisabled = []\nrate_limit = {{}}\n\n[cookie]\norigins = [\"https://bilibili.com\"]",
            detect_cpu_cores()
        );
        match fs::write(&path, default_config) {
            Ok(()) => info!(
                "Config file not found, created default config: {}",
                path.display()
            ),
            Err(e) => warn!("Failed to create default config {}: {}", path.display(), e),
        }
    }
}

fn read_config() -> Config {
    let path = match exe_config_path() {
        Some(p) => p,
        None => return Config::default(),
    };
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            warn!("Can't read {}: {}; using defaults", path.display(), e);
            return Config::default();
        }
    };
    match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            warn!("Invalid config {}: {}; using defaults", path.display(), e);
            Config::default()
        }
    }
}

fn cached() -> &'static Config {
    CONFIG.get_or_init(read_config)
}

fn exe_config_path() -> Option<std::path::PathBuf> {
    let exe = env::current_exe().ok()?;
    Some(exe.parent()?.join("config.toml"))
}

/* 检测物理/逻辑 CPU 核心数，作为默认并发基数和默认配置模板的占位值 */
fn detect_cpu_cores() -> u8 {
    match thread::available_parallelism() {
        Ok(threads) => {
            let n = threads.get().try_into().unwrap_or(u8::MAX);
            debug_assert!(n >= 1);
            n
        }
        Err(e) => {
            warn!(
                "Failed to get CPU core count, defaulting to 1 thread: {}",
                e
            );
            1
        }
    }
}

#[derive(serde::Deserialize, Clone, Default)]
struct Config {
    server: Option<ServerConfig>,
    routes: Option<RoutesConfig>,
    cookie: Option<CookieConfig>,
}

#[derive(serde::Deserialize, Clone)]
struct ServerConfig {
    port: Option<u16>,
    max_concurrent: Option<u32>,
    timeout: Option<u64>,
    /* 是否开放 /status 状态页（默认 false 关闭） */
    status_route: Option<bool>,
}

#[derive(serde::Deserialize, Clone)]
struct RoutesConfig {
    disabled: Option<Vec<String>>,
    /// 路由名 -> 缓存/限流间隔（秒）
    rate_limit: Option<HashMap<String, u64>>,
}

#[derive(serde::Deserialize, Clone)]
struct CookieConfig {
    /// 需要导出的网址列表（`rssust cookie <browser>` 会导出这些站点的 cookies）
    origins: Option<Vec<String>>,
}

pub fn server_port() -> u16 {
    cached()
        .server
        .as_ref()
        .and_then(|s: &ServerConfig| s.port)
        .unwrap_or(7878)
}

///并发上限：max_concurrent * 2（默认 CPU 核心数 * 2）
pub fn max_concurrent() -> u32 {
    let base = cached()
        .server
        .as_ref()
        .and_then(|s: &ServerConfig| s.max_concurrent)
        .unwrap_or_else(|| detect_cpu_cores() as u32);
    base.saturating_mul(2).max(1)
}

///上游请求超时（秒）
pub fn request_timeout() -> Duration {
    Duration::from_secs(
        cached()
            .server
            .as_ref()
            .and_then(|s: &ServerConfig| s.timeout)
            .unwrap_or(60),
    )
}

pub fn is_route_disabled(route: &str) -> bool {
    cached()
        .routes
        .as_ref()
        .and_then(|r: &RoutesConfig| r.disabled.as_ref())
        .is_some_and(|v| v.iter().any(|d| d == route))
}

/// 该路由的上游响应缓存间隔（秒）；未配置则返回 None（不缓存）。
pub fn rate_limit_secs(route: &str) -> Option<u64> {
    cached()
        .routes
        .as_ref()
        .and_then(|r: &RoutesConfig| r.rate_limit.as_ref())
        .and_then(|m| m.get(route).copied())
}

/// `rssust cookie` 需要导出的网址列表（`[cookie].origins`）。
pub fn cookie_origins() -> Vec<String> {
    cached()
        .cookie
        .as_ref()
        .and_then(|c: &CookieConfig| c.origins.as_ref())
        .cloned()
        .unwrap_or_default()
}

/// 是否开放 `/status` 状态页（`[server] status_route`，默认 false=关闭）
pub fn status_route_enabled() -> bool {
    cached()
        .server
        .as_ref()
        .and_then(|s: &ServerConfig| s.status_route)
        .unwrap_or(false)
}
