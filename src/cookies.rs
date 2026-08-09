use cookie_scoop::{get_cookies, BrowserName, CookieSameSite, GetCookiesOptions};
use std::fs;
use std::io;
use std::path::PathBuf;

fn read_existing_cookies(path: &PathBuf) -> Vec<serde_json::Value> {
    if !path.exists() {
        return Vec::new();
    }
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    if text.trim().is_empty() {
        return Vec::new();
    }
    serde_json::from_str(&text).unwrap_or_default()
}

pub async fn extract_cookies_to_json(target_browser: BrowserName) -> Result<(), io::Error> {
    let exe_path = std::env::current_exe().map_err(io::Error::other)?;
    let exe_dir = exe_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not get executable directory",
        )
    })?;
    let output_path = exe_dir.join("cookies.json");

    let existing = read_existing_cookies(&output_path);

    let mut origins = crate::config::cookie_origins();
    for c in &existing {
        if let Some(domain) = c.get("domain").and_then(|v| v.as_str()) {
            origins.push(format!("https://{}", domain.trim_start_matches(".")));
        }
    }
    origins.dedup();
    if origins.is_empty() {
        origins.push("https://bilibili.com".to_string());
    }
    let url = origins[0].clone();

    let result = get_cookies(
        GetCookiesOptions::new(url)
            .browsers(vec![target_browser])
            .origins(origins),
    )
    .await;

    for w in &result.warnings {
        eprintln!("warning: {w}");
    }

    if result.cookies.is_empty() && !result.warnings.is_empty() {
        return Err(io::Error::other(format!(
            "Failed to find cookies: {}",
            result.warnings.join("; ")
        )));
    }

    let mut merged = existing;
    for cookie in result.cookies {
        let json_cookie = serde_json::json!({
            "name": cookie.name,
            "value": cookie.value,
            "domain": cookie.domain.unwrap_or_default(),
            "path": cookie.path.unwrap_or_default(),
            "secure": cookie.secure.unwrap_or(false),
            "httpOnly": cookie.http_only.unwrap_or(false),
            "sameSite": match cookie.same_site {
                Some(CookieSameSite::Strict) => "Strict",
                Some(CookieSameSite::Lax) => "Lax",
                Some(CookieSameSite::None) => "None",
                _ => "Unspecified",
            },
            "expirationDate": cookie.expires,
        });

        let replace_idx = merged.iter().position(|c| {
            c.get("name").and_then(|v| v.as_str()) == Some(json_cookie["name"].as_str().unwrap_or_default())
                && c.get("domain").and_then(|v| v.as_str()) == Some(json_cookie["domain"].as_str().unwrap_or_default())
                && c.get("path").and_then(|v| v.as_str()) == Some(json_cookie["path"].as_str().unwrap_or_default())
        });

        match replace_idx {
            Some(idx) => merged[idx] = json_cookie,
            None => merged.push(json_cookie),
        }
    }

    let final_json = serde_json::to_string_pretty(&merged)
        .map_err(io::Error::other)?;

    fs::write(&output_path, final_json)?;
    println!("Download cookies from {:?} done", target_browser);
    Ok(())
}
