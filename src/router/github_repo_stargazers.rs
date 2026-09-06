use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::{API, owner_repo, rest_get_with_accept};
///GitHub 仓库加星用户时间线，经 REST `GET /repos/{owner}/{repo}/stargazers` 获取
///（`application/vnd.github.star+json` 媒体类型以携带 starred_at 时间）。
///Params: owner, repo, limit (默认30, 最大100)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let (owner, repo) = owner_repo(&para)?;
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(30)
        .min(100);

    let url = format!(
        "{}/repos/{}/{}/stargazers?per_page={}",
        API, owner, repo, limit
    );

    let json: Value = rest_get_with_accept(&url, "application/vnd.github.star+json").await?;
    let stargazers = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    if stargazers.is_empty() {
        return Err(anyhow!("No stargazers found. Check owner/repo parameters"));
    }

    let mut item_vec = Vec::new();
    for s in stargazers {
        let user = &s["user"];
        let login = user["login"].as_str().unwrap_or("");
        let avatar = user["avatar_url"].as_str().unwrap_or("");
        let html_url = user["html_url"].as_str().unwrap_or("");
        let starred = s["starred_at"].as_str().unwrap_or("");

        let title = format!("{} starred {}/{}", login, owner, repo);
        let desc = format!(
            "<img src=\"{}\" referrerpolicy=\"no-referrer\"><p><a href=\"{}\">{}</a></p><p>Star 时间: {}</p>",
            escape_html(avatar),
            escape_html(html_url),
            escape_html(login),
            starred
        );

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(html_url.to_string())
            .description(Some(desc))
            .pub_date(rfc3339_to_rss(starred))
            .author(Some(login.to_string()))
            .guid(rss::Guid {
                value: format!(
                    "{}@{}",
                    user["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                    starred
                ),
                permalink: false,
            })
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Stargazers - {}/{}", owner, repo))
        .link(format!("https://github.com/{}/{}", owner, repo))
        .description(format!("Users who starred {}/{}", owner, repo))
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