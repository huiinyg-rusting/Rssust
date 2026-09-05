use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::{API, rest_get};
///GitHub repository search via REST API `GET /search/repositories`.
///Params: q (query, required), sort (stars/forks/updated/help-wanted-issues/best-match, default stars), order (asc/desc, default desc), limit (default 20, max 50)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let q = para
        .get("q")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("Missing q parameter (search query)"))?;
    let sort = para.get("sort").cloned().unwrap_or_else(|| "stars".to_string());
    let order = para.get("order").cloned().unwrap_or_else(|| "desc".to_string());
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(50);

    let url = format!(
        "{}/search/repositories?q={}&sort={}&order={}&per_page={}",
        API,
        urlencoding::encode(&q),
        sort,
        order,
        limit
    );

    let json: Value = rest_get(&url).await?;
    if json["message"].as_str().is_some() {
        return Err(anyhow!(
            "GitHub API error: {}",
            json["message"].as_str().unwrap_or("")
        ));
    }

    let items = json["items"]
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    if items.is_empty() {
        return Err(anyhow!("No repositories found for this query"));
    }

    let mut item_vec = Vec::new();
    for r in items {
        let full_name = r["full_name"].as_str().unwrap_or("");
        let html_url = r["html_url"].as_str().unwrap_or("");
        let description = r["description"].as_str().unwrap_or("").to_string();
        let language = r["language"].as_str().unwrap_or("");
        let stars = r["stargazers_count"].as_i64().unwrap_or(0);
        let forks = r["forks_count"].as_i64().unwrap_or(0);
        let issues = r["open_issues_count"].as_i64().unwrap_or(0);
        let updated_at = r["updated_at"].as_str().unwrap_or("");
        let owner = r["owner"]["login"].as_str().unwrap_or("");

        let mut topics: Vec<String> = Vec::new();
        if let Some(arr) = r["topics"].as_array() {
            for t in arr {
                if let Some(n) = t.as_str() {
                    topics.push(n.to_string());
                }
            }
        }

        let mut desc = String::new();
        if !description.is_empty() {
            desc.push_str(&format!("<p>{}</p>", escape_html(&description)));
        }
        let mut meta = Vec::new();
        if !language.is_empty() {
            meta.push(format!("Language: {}", escape_html(language)));
        }
        meta.push(format!("Stars: {}<br>Forks: {}<br>Issues: {}", stars, forks, issues));
        if !topics.is_empty() {
            meta.push(format!("Topics: {}", topics.join(", ")));
        }
        meta.push(format!("Updated: {}", updated_at));
        desc.push_str(&format!("<p>{}</p>", meta.join("<br>")));

        let item = ItemBuilder::default()
            .title(Some(full_name.to_string()))
            .link(html_url.to_string())
            .description(if desc.is_empty() { None } else { Some(desc) })
            .pub_date(rfc3339_to_rss(updated_at))
            .author(Some(owner.to_string()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Search - {}", q))
        .link("https://github.com/search?type=repositories".to_string())
        .description(format!("GitHub repository search: {} ({}, {})", q, sort, order))
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