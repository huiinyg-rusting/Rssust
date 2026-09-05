use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::{API, owner_repo, rest_get};
///GitHub repository Branches via REST API `GET /repos/{owner}/{repo}/branches`.
///Params: owner, repo, limit (default 30, max 100)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let (owner, repo) = owner_repo(&para)?;
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(30)
        .min(100);

    let url = format!("{}/repos/{}/{}/branches?per_page={}", API, owner, repo, limit);

    let json: Value = rest_get(&url).await?;
    let branches = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    if branches.is_empty() {
        return Err(anyhow!("No branches found. Check owner/repo parameters"));
    }

    let mut item_vec = Vec::new();
    for b in branches {
        let name = b["name"].as_str().unwrap_or("");
        let sha = b["commit"]["sha"].as_str().unwrap_or("");
        let short = if sha.len() >= 7 { &sha[..7] } else { sha };
        let protected = b["protected"].as_bool().unwrap_or(false);

        let description = format!(
            "commit: {}<br>protected: {}<br><a href=\"https://github.com/{}/{}/tree/{}\">查看代码</a>",
            short,
            if protected { "是" } else { "否" },
            owner,
            repo,
            name
        );

        let item = ItemBuilder::default()
            .title(Some(name.to_string()))
            .link(format!("https://github.com/{}/{}/tree/{}", owner, repo, name))
            .description(Some(description))
            .pub_date(now())
            .guid(rss::Guid {
                value: format!("https://github.com/{}/{}/branch/{}", owner, repo, name),
                permalink: false,
            })
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Branches - {}/{}", owner, repo))
        .link(format!("https://github.com/{}/{}/branches", owner, repo))
        .description(format!("GitHub repository {}/{} branches", owner, repo))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}