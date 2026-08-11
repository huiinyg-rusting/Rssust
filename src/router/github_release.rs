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

///GitHub repository Releases via REST API `GET /repos/{owner}/{repo}/releases`.
///Params: owner, repo, limit (default 20, max 50)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let owner = para
        .get("owner")
        .cloned()
        .ok_or_else(|| anyhow!("Missing owner parameter (repository owner)"))?;
    let repo = para
        .get("repo")
        .cloned()
        .ok_or_else(|| anyhow!("Missing repo parameter (repository name)"))?;
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(50);

    let token = token()?;
    let url = format!("{}/repos/{}/{}/releases?per_page={}", API, owner, repo, limit);

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
            .pub_date(parse_github_date(published_at))
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