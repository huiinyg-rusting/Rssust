use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const API: &str = "https://api.github.com";

fn token() -> Result<String> {
    env_search("GITHUB_TOKEN").ok_or_else(|| {
        anyhow!("Environment variable GITHUB_TOKEN is required (GitHub PAT). See docs.")
    })
}

fn parse_github_date(s: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
        .unwrap_or_else(now)
}

///GitHub user starred repositories via REST API `GET /users/{username}/starred`.
///Params: username, limit (default 20, max 100)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let username = para
        .get("username")
        .cloned()
        .ok_or_else(|| anyhow!("Missing username parameter (GitHub login)"))?;
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(100);

    let token = token()?;
    let url = format!(
        "{}/users/{}/starred?sort=created&direction=desc&per_page={}",
        API, username, limit
    );

    let resp = fetch_reqwest_get_with_headers(
        &url,
        &[
            ("Authorization", &format!("Bearer {}", token)),
            ("User-Agent", "rssust-github-router/1.0"),
            ("Accept", "application/vnd.github+json"),
        ],
    )
    .await?;

    let json: Value = serde_json::from_str(&resp)?;
    let repos = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    if repos.is_empty() {
        return Err(anyhow!("No starred repositories found. Check username parameter"));
    }

    let mut item_vec = Vec::new();
    for r in repos {
        let full_name = r["full_name"].as_str().unwrap_or("");
        let html_url = r["html_url"].as_str().unwrap_or("");
        let description = r["description"].as_str().unwrap_or("").to_string();
        let language = r["language"].as_str().unwrap_or("");
        let stars = r["stargazers_count"].as_i64().unwrap_or(0);
        let forks = r["forks_count"].as_i64().unwrap_or(0);
        let pushed_at = r["pushed_at"].as_str().unwrap_or("");

        let mut desc = String::new();
        if !description.is_empty() {
            desc.push_str(&format!("<p>{}</p>", escape_html(&description)));
        }
        let mut meta = Vec::new();
        if !language.is_empty() {
            meta.push(format!("Language: {}", escape_html(language)));
        }
        meta.push(format!("Stars: {}<br>Forks: {}", stars, forks));
        meta.push(format!("Updated: {}", pushed_at));
        desc.push_str(&format!("<p>{}</p>", meta.join("<br>")));

        let item = ItemBuilder::default()
            .title(Some(full_name.to_string()))
            .link(html_url.to_string())
            .description(if desc.is_empty() { None } else { Some(desc) })
            .pub_date(parse_github_date(pushed_at))
            .author(Some(username.clone()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Starred - {}", username))
        .link(format!("https://github.com/{}?tab=stars", username))
        .description(format!("Repositories starred by GitHub user {}", username))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}