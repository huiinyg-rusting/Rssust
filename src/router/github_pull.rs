use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::{API, owner_repo, rest_get};
///GitHub repository Pull Requests via REST API `GET /repos/{owner}/{repo}/pulls`.
///Params: owner, repo, state (open/closed/all, default open), limit (default 20, max 50)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let (owner, repo) = owner_repo(&para)?;
    let state = para.get("state").cloned().unwrap_or_else(|| "open".to_string());
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(50);

    let url = format!(
        "{}/repos/{}/{}/pulls?state={}&per_page={}",
        API, owner, repo, state, limit
    );

    let json: Value = rest_get(&url).await?;
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
            .pub_date(rfc3339_to_rss(created_at))
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