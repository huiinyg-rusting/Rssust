use std::env;
use std::fs;
use std::thread;
use tracing::{info, warn};

pub fn init() {
    let path = match exe_config_path() {
        Some(p) => p,
        None => return,
    };
    if !path.exists() {
        let default = format!(
            "[server]\nport = 7878\n\n[routes]\ndisabled = []\nnumofcore = {}",
            realcorenum()
        );
        match fs::write(&path, default) {
            Ok(()) => info!("Config file not found, created default config: {}", path.display()),
            Err(e) => warn!("Failed to create default config {}: {}", path.display(), e),
        }
    }
}

fn load_config() -> Config {
    let path = exe_config_path().expect("Can't find Config.toml");
    let content = fs::read_to_string(&path).expect("Can't read Config.toml");
    let config: Config = toml::from_str(&content).expect("Can't Toml the config string");
    config
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
            warn!("Failed to get CPU core count, defaulting to 1 thread: {}", e);
            1
        },
    }
}

#[derive(serde::Deserialize)]
struct Config {
    server: Option<ServerConfig>,
    routes: Option<RoutesConfig>,
}

#[derive(serde::Deserialize)]
struct ServerConfig {
    port: Option<u16>,
}

#[derive(serde::Deserialize)]
struct RoutesConfig {
    disabled: Option<Vec<String>>,
    numofcore: Option<u8>,
}

pub fn server_port() -> u16 {
    load_config()
        .server
        .and_then(|s: ServerConfig| s.port)
        .unwrap_or(7878)
}

pub fn is_route_disabled(route: &str) -> bool {
    load_config()
        .routes
        .and_then(|r: RoutesConfig| r.disabled)
        .map_or(false, |v| v.contains(&route.to_string()))
}

pub fn numofcore() -> u8 {
    load_config()
        .routes
        .and_then(|r: RoutesConfig| r.numofcore)
        .unwrap_or_else(|| realcorenum())

}

