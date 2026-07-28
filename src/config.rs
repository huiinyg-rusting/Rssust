use std::env;
use std::fs;

pub fn init() {
    let path = match exe_config_path() {
        Some(p) => p,
        None => return,
    };
    if !path.exists() {
        let _ = fs::write(&path, "[routes]\ndisabled = []\n");
    }
}

pub fn is_route_disabled(route: &str) -> bool {
    let path = match exe_config_path() {
        Some(p) => p,
        None => return false,
    };
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let config: Config = match toml::from_str(&content) {
        Ok(c) => c,
        Err(_) => return false,
    };
    config
        .routes
        .and_then(|r| r.disabled)
        .map_or(false, |v| v.contains(&route.to_string()))
}

fn exe_config_path() -> Option<std::path::PathBuf> {
    let exe = env::current_exe().ok()?;
    Some(exe.parent()?.join("config.toml"))
}

#[derive(serde::Deserialize)]
struct Config {
    routes: Option<RoutesConfig>,
}

#[derive(serde::Deserialize)]
struct RoutesConfig {
    disabled: Option<Vec<String>>,
}
