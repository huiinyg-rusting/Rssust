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

///GitHub repository Tags via REST API `GET /repos/{owner}/{repo}/tags`.
///Params: owner, repo, limit (default 30, max 100)
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
        .unwrap_or(30)
        .min(100);

    let token = token()?;
    let url = format!("{}/repos/{}/{}/tags?per_page={}", API, owner, repo, limit);

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
    let tags = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    if tags.is_empty() {
        return Err(anyhow!("No tags found. Check owner/repo parameters"));
    }

    let mut item_vec = Vec::new();
    for t in tags {
        let name = t["name"].as_str().unwrap_or("");
        let sha = t["commit"]["sha"].as_str().unwrap_or("");
        let short = if sha.len() >= 7 { &sha[..7] } else { sha };

        let description = format!(
            "commit: {}<br><a href=\"https://github.com/{}/{}/tree/{}\">查看代码</a>",
            short, owner, repo, name
        );

        let item = ItemBuilder::default()
            .title(Some(name.to_string()))
            .link(format!("https://github.com/{}/{}/releases/tag/{}", owner, repo, name))
            .description(Some(description))
            .pub_date(now())
            .guid(rss::Guid {
                value: format!("https://github.com/{}/{}/tree/{}", owner, repo, name),
                permalink: false,
            })
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Tags - {}/{}", owner, repo))
        .link(format!("https://github.com/{}/{}/tags", owner, repo))
        .description(format!("GitHub repository {}/{} tags", owner, repo))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}