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

///GitHub 用户的 Gist 列表，经 REST `GET /users/{username}/gists` 获取。
///Params: username, limit (default 20, max 100)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let username = para
        .get("username")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("Missing username parameter (GitHub username)"))?;
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(100);

    let token = token()?;
    let url = format!("{}/users/{}/gists?per_page={}", API, username, limit);

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
    let gists = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    let mut item_vec = Vec::new();
    for g in gists {
        let id = g["id"].as_str().unwrap_or("");
        let description = g["description"].as_str().unwrap_or("");
        let html_url = g["html_url"].as_str().unwrap_or("");
        let created_at = g["created_at"].as_str().unwrap_or("");
        let updated_at = g["updated_at"].as_str().unwrap_or("");
        let comments = g["comments"].as_i64().unwrap_or(0);
        let fork = g["fork"].as_bool().unwrap_or(false);

        let mut files_desc = Vec::new();
        let mut first_file = String::new();
        if let Some(files) = g["files"].as_object() {
            for (name, f) in files {
                let language = f["language"].as_str().unwrap_or("");
                let size = f["size"].as_i64().unwrap_or(0);
                let raw = f["raw_url"].as_str().unwrap_or("");
                if first_file.is_empty() {
                    first_file = name.clone();
                }
                if language.is_empty() {
                    files_desc.push(format!("{} ({:.1} KB)", name, size as f64 / 1024.0));
                } else {
                    files_desc.push(format!("{} [{}] ({:.1} KB)", name, language, size as f64 / 1024.0));
                }
                files_desc.push(format!("原始: <a href=\"{}\">{}</a>", raw, name));
            }
        }

        let mut desc = format!(
            "创建: {}<br>更新: {}<br>评论数: {}{}",
            parse_github_date(created_at),
            parse_github_date(updated_at),
            comments,
            if fork { "<br>状态: fork(分叉)" } else { "" }
        );
        if !description.is_empty() {
            desc.push_str(&format!("<br><pre>{}</pre>", escape_html(description)));
        }
        if !files_desc.is_empty() {
            desc.push_str(&format!("<br>文件: {}", files_desc.join("<br>")));
        }

        let item = ItemBuilder::default()
            .title(Some(format!(
                "{} - gist {}",
                username,
                if first_file.is_empty() { id } else { &first_file }
            )))
            .link(html_url.to_string())
            .description(Some(desc))
            .pub_date(parse_github_date(updated_at))
            .guid(rss::Guid {
                value: html_url.to_string(),
                permalink: false,
            })
            .build();
        item_vec.push(item);
    }

    if item_vec.is_empty() {
        return Err(anyhow!("No gists found for user {}", username));
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Gists - {}", username))
        .link(format!("https://gist.github.com/{}", username))
        .description(format!("GitHub user {}'s public gists", username))
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