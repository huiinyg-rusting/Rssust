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

///GitHub 仓库贡献者列表，经 REST `GET /repos/{owner}/{repo}/contributors` 获取。
///Params: owner, repo, order (desc/asc, 默认desc), anon (1=包含匿名贡献者), limit (默认30, 最大100)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let owner = para
        .get("owner")
        .cloned()
        .ok_or_else(|| anyhow!("Missing owner parameter (repository owner)"))?;
    let repo = para
        .get("repo")
        .cloned()
        .ok_or_else(|| anyhow!("Missing repo parameter (repository name)"))?;
    let order = para
        .get("order")
        .cloned()
        .unwrap_or_else(|| "desc".to_string());
    let anon = parse_bool(para.get("anon"), false);
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(30)
        .min(100);

    let token = token()?;
    let mut url = format!(
        "{}/repos/{}/{}/contributors?per_page={}",
        API, owner, repo, limit
    );
    if anon {
        url.push_str("&anon=1");
    }

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
    let mut contributors = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?
        .to_vec();

    contributors.sort_by(|a, b| {
        a["contributions"]
            .as_i64()
            .unwrap_or(0)
            .cmp(&b["contributions"].as_i64().unwrap_or(0))
    });
    if order != "asc" {
        contributors.reverse();
    }

    if contributors.is_empty() {
        return Err(anyhow!("No contributors found. Check owner/repo parameters"));
    }

    let mut item_vec = Vec::new();
    for c in contributors {
        let contributions = c["contributions"].as_i64().unwrap_or(0);
        let typ = c["type"].as_str().unwrap_or("User");

        let (title, desc, link, guid) = if typ == "Anonymous" {
            let name = c["name"].as_str().unwrap_or("");
            let email = c["email"].as_str().unwrap_or("");
            (
                format!("Contributor: {}", name),
                format!(
                    "<p>匿名贡献者</p><p>名称: {}</p><p>邮箱: {}</p><p>贡献次数: {}</p>",
                    escape_html(name),
                    escape_html(email),
                    contributions
                ),
                String::new(),
                format!("anon-{}", escape_html(name)),
            )
        } else {
            let login = c["login"].as_str().unwrap_or("");
            let avatar = c["avatar_url"].as_str().unwrap_or("");
            let html_url = c["html_url"].as_str().unwrap_or("");
            (
                format!("Contributor: {}", login),
                format!(
                    "<img src=\"{}\" referrerpolicy=\"no-referrer\"><p><a href=\"{}\">{}</a></p><p>贡献次数: {}</p>",
                    avatar, html_url, login, contributions
                ),
                html_url.to_string(),
                c["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
            )
        };

        let link_opt = if link.is_empty() { None } else { Some(link) };
        let item = ItemBuilder::default()
            .title(Some(title))
            .link(link_opt)
            .description(Some(desc))
            .author(Some(format!("contributions: {}", contributions)))
            .guid(rss::Guid {
                value: guid,
                permalink: false,
            })
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Contributors - {}/{}", owner, repo))
        .link(format!("https://github.com/{}/{}/graphs/contributors", owner, repo))
        .description(format!("New contributors for {}/{}", owner, repo))
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