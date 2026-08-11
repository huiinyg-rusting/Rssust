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

///GitHub repository Pull Requests via REST API `GET /repos/{owner}/{repo}/pulls`.
///Params: owner, repo, state (open/closed/all, default open), limit (default 20, max 50)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let owner = para
        .get("owner")
        .cloned()
        .ok_or_else(|| anyhow!("Missing owner parameter (repository owner)"))?;
    let repo = para
        .get("repo")
        .cloned()
        .ok_or_else(|| anyhow!("Missing repo parameter (repository name)"))?;
    let state = para.get("state").cloned().unwrap_or_else(|| "open".to_string());
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(50);

    let token = token()?;
    let url = format!(
        "{}/repos/{}/{}/pulls?state={}&per_page={}",
        API, owner, repo, state, limit
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
    let pulls = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    if pulls.is_empty() {
        return Err(anyhow!("No pull requests found. Check owner/repo parameters"));
    }

    let mut item_vec = Vec::new();
    for pr in pulls {
        let number = pr["number"].as_i64().unwrap_or(0);
        let title = pr["title"].as_str().unwrap_or("").to_string();
        let user = pr["user"]["login"].as_str().unwrap_or("");
        let html_url = pr["html_url"].as_str().unwrap_or("");
        let created_at = pr["created_at"].as_str().unwrap_or("");
        let body = pr["body"].as_str().unwrap_or("").trim().to_string();
        let base_ref = pr["base"]["ref"].as_str().unwrap_or("");
        let head_ref = pr["head"]["ref"].as_str().unwrap_or("");

        let mut desc = format!("作者: {}<br>分支: {} -> {}", user, head_ref, base_ref);
        if let Some(commits) = pr["commits"].as_i64() {
            desc.push_str(&format!("<br>Commits: {}", commits));
        }
        if !body.is_empty() {
            desc.push_str(&format!(
                "<br><pre>{}</pre>",
                escape_html(&body.chars().take(2000).collect::<String>())
            ));
        }

        let item = ItemBuilder::default()
            .title(Some(format!("#{} {}", number, title)))
            .link(html_url.to_string())
            .description(Some(desc))
            .pub_date(parse_github_date(created_at))
            .author(Some(user.to_string()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Pull Requests - {}/{}", owner, repo))
        .link(format!("https://github.com/{}/{}/pulls", owner, repo))
        .description(format!("GitHub repository {}/{} pull requests", owner, repo))
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