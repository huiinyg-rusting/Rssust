use bench_scraper::{self, KnownBrowser, SameSite};
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

pub fn extract_cookies_to_json(target_browser: KnownBrowser) -> Result<(), io::Error> {
    let exe_path = std::env::current_exe().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let exe_dir = exe_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not get executable directory",
        )
    })?;
    let output_path = exe_dir.join("cookies.json");

    let all_sessions = bench_scraper::find_cookies().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to find cookies: {:?}", e),
        )
    })?;

    let mut existing = read_existing_cookies(&output_path);
    let mut found = false;

    for session in all_sessions {
        if session.browser == target_browser {
            found = true;
            for cookie in session.cookies {
                let exp_ts = cookie.expiration_time.map(|dt| dt.unix_timestamp() as f64);

                let json_cookie = serde_json::json!({
                    "name": cookie.name,
                    "value": cookie.value,
                    "domain": cookie.host,
                    "path": cookie.path,
                    "secure": cookie.is_secure,
                    "httpOnly": cookie.is_http_only,
                    "sameSite": match cookie.same_site {
                        Some(SameSite::Strict) => "Strict",
                        Some(SameSite::Lax) => "Lax",
                        Some(SameSite::None) => "None",
                        _ => "Unspecified",
                    },
                    "expirationDate": exp_ts,
                });

                let replace_idx = existing.iter().position(|c| {
                    c.get("name").and_then(|v| v.as_str()) == Some(&cookie.name)
                        && c.get("domain").and_then(|v| v.as_str()) == Some(&cookie.host)
                        && c.get("path").and_then(|v| v.as_str()) == Some(&cookie.path)
                });

                match replace_idx {
                    Some(idx) => existing[idx] = json_cookie,
                    None => existing.push(json_cookie),
                }
            }
        }
    }

    if !found {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Browser not found"));
    }

    let final_json = serde_json::to_string_pretty(&existing)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(&output_path, final_json)?;
    println!("Download cookies from {:?} done", target_browser);
    Ok(())
}
