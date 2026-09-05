use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::{API, owner_repo, rest_get};
///GitHub repository Releases via REST API `GET /repos/{owner}/{repo}/releases`.
///Params: owner, repo, limit (default 20, max 50)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let (owner, repo) = owner_repo(&para)?;
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(50);

    let url = format!("{}/repos/{}/{}/releases?per_page={}", API, owner, repo, limit);

    let json: Value = rest_get(&url).await?;
    let releases = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    if releases.is_empty() {
        return Err(anyhow!("No releases found. Check owner/repo parameters"));
    }

    let mut item_vec = Vec::new();
    for r in releases {
        let tag = r["tag_name"].as_str().unwrap_or("");
        let name = r["name"].as_str().unwrap_or("");
        let author = r["author"]["login"].as_str().unwrap_or("");
        let html_url = r["html_url"].as_str().unwrap_or("");
        let published_at = r["published_at"].as_str().unwrap_or("");
        let draft = r["draft"].as_bool().unwrap_or(false);
        let prerelease = r["prerelease"].as_bool().unwrap_or(false);
        let body = r["body"].as_str().unwrap_or("").trim().to_string();

        let title = if name.is_empty() {
            tag.to_string()
        } else {
            format!("{} - {}", tag, name)
        };

        let mut desc = format!("作者: {}<br>标签: {}", author, tag);
        if draft {
            desc.push_str("<br>状态: draft(草稿)");
        }
        if prerelease {
            desc.push_str("<br>状态: prerelease(预发布)");
        }
        if !body.is_empty() {
            desc.push_str(&format!(
                "<br><pre>{}</pre>",
                escape_html(&body.chars().take(2000).collect::<String>())
            ));
        }

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(html_url.to_string())
            .description(Some(desc))
            .pub_date(rfc3339_to_rss(published_at))
            .author(Some(author.to_string()))
            .guid(rss::Guid {
                value: html_url.to_string(),
                permalink: false,
            })
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Releases - {}/{}", owner, repo))
        .link(format!("https://github.com/{}/{}/releases", owner, repo))
        .description(format!("GitHub repository {}/{} releases", owner, repo))
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