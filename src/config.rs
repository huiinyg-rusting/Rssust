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
        let default = format!(
            "[server]\nport = 7878\nmax_concurrent = {}\ntimeout = 60\n\n[routes]\ndisabled = []",
            realcorenum()
        );
        match fs::write(&path, default) {
            Ok(()) => info!(
                "Config file not found, created default config: {}",
                path.display()
            ),
            Err(e) => warn!("Failed to create default config {}: {}", path.display(), e),
        }
    }
    let _ = load_config();
}

fn load_config() -> Config {
    let path = exe_config_path().expect("Can't find Config.toml");
    let content = fs::read_to_string(&path).expect("Can't read Config.toml");
    let config: Config = toml::from_str(&content).expect("Can't Toml the config string");
    let _ = CONFIG.set(config.clone());
    config
}

fn cached() -> &'static Config {
    CONFIG.get_or_init(|| {
        let path = exe_config_path().expect("Can't find Config.toml");
        let content = fs::read_to_string(&path).expect("Can't read Config.toml");
        toml::from_str(&content).expect("Can't Toml the config string")
    })
}

fn exe_config_path() -> Option<std::path::PathBuf> {
    let exe = env::current_exe().ok()?;
    Some(exe.parent()?.join("config.toml"))
}

fn realcorenum() -> u8 {
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

#[derive(serde::Deserialize, Clone)]
struct Config {
    server: Option<ServerConfig>,
    routes: Option<RoutesConfig>,
}

#[derive(serde::Deserialize, Clone)]
struct ServerConfig {
    port: Option<u16>,
    max_concurrent: Option<u32>,
    timeout: Option<u64>,
}

#[derive(serde::Deserialize, Clone)]
struct RoutesConfig {
    disabled: Option<Vec<String>>,
    /// 路由名 -> 缓存/限流间隔（秒）
    rate_limit: Option<HashMap<String, u64>>,
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
        .unwrap_or_else(|| realcorenum() as u32);
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
        .map_or(false, |v| v.iter().any(|d| d == route))
}

/// 该路由的上游响应缓存间隔（秒）；未配置则返回 None（不缓存）。
pub fn rate_limit_secs(route: &str) -> Option<u64> {
    cached()
        .routes
        .as_ref()
        .and_then(|r: &RoutesConfig| r.rate_limit.as_ref())
        .and_then(|m| m.get(route).copied())
}
