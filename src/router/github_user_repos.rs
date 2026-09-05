use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::{API, rest_get};
///GitHub user repositories via REST API `GET /users/{username}/repos`.
///Params: username, type (all/owner/member, default owner), limit (default 20, max 100)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let username = para
        .get("username")
        .cloned()
        .ok_or_else(|| anyhow!("Missing username parameter (GitHub login)"))?;
    let repo_type = para.get("type").cloned().unwrap_or_else(|| "owner".to_string());
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(100);

    let url = format!(
        "{}/users/{}/repos?type={}&sort=updated&per_page={}",
        API, username, repo_type, limit
    );

    let json: Value = rest_get(&url).await?;
    let repos = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    if repos.is_empty() {
        return Err(anyhow!("No repositories found. Check username parameter"));
    }

    let mut item_vec = Vec::new();
    for r in repos {
        let full_name = r["full_name"].as_str().unwrap_or("");
        let html_url = r["html_url"].as_str().unwrap_or("");
        let description = r["description"].as_str().unwrap_or("").to_string();
        let language = r["language"].as_str().unwrap_or("");
        let stars = r["stargazers_count"].as_i64().unwrap_or(0);
        let forks = r["forks_count"].as_i64().unwrap_or(0);
        let fork = r["fork"].as_bool().unwrap_or(false);
        let archived = r["archived"].as_bool().unwrap_or(false);
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
        if fork {
            meta.push("fork".to_string());
        }
        if archived {
            meta.push("archived".to_string());
        }
        meta.push(format!("Updated: {}", pushed_at));
        desc.push_str(&format!("<p>{}</p>", meta.join("<br>")));

        let item = ItemBuilder::default()
            .title(Some(full_name.to_string()))
            .link(html_url.to_string())
            .description(if desc.is_empty() { None } else { Some(desc) })
            .pub_date(rfc3339_to_rss(pushed_at))
            .author(Some(username.clone()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Repositories - {}", username))
        .link(format!("https://github.com/{}", username))
        .description(format!("GitHub user {} repositories", username))
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