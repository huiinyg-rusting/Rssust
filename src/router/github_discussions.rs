use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const GRAPHQL: &str = "https://api.github.com/graphql";

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

///GitHub 仓库 Discussion 列表（GraphQL，需 token）。
///Params: owner, repo, state (open/closed/answered/unanswered/locked/unlocked/all), category(分类名), limit (默认20, 最大100)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let owner = para
        .get("owner")
        .cloned()
        .ok_or_else(|| anyhow!("Missing owner parameter (repository owner)"))?;
    let repo = para
        .get("repo")
        .cloned()
        .ok_or_else(|| anyhow!("Missing repo parameter (repository name)"))?;
    let state = para
        .get("state")
        .cloned()
        .unwrap_or_else(|| "open".to_string());
    let category = para.get("category").cloned().filter(|s| !s.is_empty());
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(100);

    let token = token()?;
    let auth = format!("Bearer {}", token);
    let headers: [(&str, &str); 3] = [
        ("Authorization", auth.as_str()),
        ("User-Agent", "rssust-github-router/1.0"),
        ("Accept", "application/vnd.github+json"),
    ];

    let owner_q = serde_json::to_string(&owner).unwrap_or_else(|_| format!("\"{}\"", owner));
    let repo_q = serde_json::to_string(&repo).unwrap_or_else(|_| format!("\"{}\"", repo));

    // GitHub GraphQL 的 discussions 连接参数是 states: [DiscussionState]（OPEN/CLOSED/LOCKED/
    // UNLOCKED/ANSWERED/UNANSWERED），不存在 closed/locked/answered 布尔参数。
    let mut filters = format!("first: {}", limit);
    let state_enum: Option<&str> = match state.as_str() {
        "answered" => Some("ANSWERED"),
        "unanswered" => Some("UNANSWERED"),
        "closed" => Some("CLOSED"),
        "open" => Some("OPEN"),
        "locked" => Some("LOCKED"),
        "unlocked" => Some("UNLOCKED"),
        "all" => None,
        _ => return Err(anyhow!("state 参数仅支持 open/closed/answered/unanswered/locked/unlocked/all")),
    };
    if let Some(s) = state_enum {
        filters.push_str(&format!(", states: [{}]", s));
    }

    if let Some(cat) = &category {
        let cat_query = format!(
            "{{ repository(owner: {}, name: {}) {{ discussionCategories(first: 25) {{ nodes {{ id name }} }} }} }}",
            owner_q, repo_q
        );
        let cat_body = serde_json::json!({ "query": cat_query }).to_string();
        let cat_resp = fetch_reqwest_post_json_with_headers(GRAPHQL, &cat_body, &headers).await?;
        let cat_json: Value = serde_json::from_str(&cat_resp)?;
        let mut cat_id: Option<String> = None;
        if let Some(nodes) = cat_json
            .pointer("/data/repository/discussionCategories/nodes")
            .and_then(Value::as_array)
        {
            for n in nodes {
                if n["name"].as_str() == Some(cat.as_str()) {
                    cat_id = n["id"].as_str().map(String::from);
                    break;
                }
            }
        }
        if let Some(id) = cat_id {
            filters.push_str(&format!(", categoryId: \"{}\"", id));
        }
    }

    let query = format!(
        "{{ repository(owner: {}, name: {}) {{ discussions({}) {{ nodes {{ title author {{ login }} createdAt closed isAnswered locked body url }} }} }} }}",
        owner_q, repo_q, filters
    );

    // GitHub GraphQL 要求 body 为 JSON: {"query": "..."}，不能直接发送原始查询字符串
    let query_body = serde_json::json!({ "query": query }).to_string();
    let resp = fetch_reqwest_post_json_with_headers(GRAPHQL, &query_body, &headers).await?;
    let json: Value = serde_json::from_str(&resp)?;

    let nodes = json
        .pointer("/data/repository/discussions/nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("GraphQL 返回缺少 data.repository.discussions.nodes"))?;

    let mut item_vec = Vec::new();
    for d in nodes {
        let title = d["title"].as_str().unwrap_or("");
        let author = d["author"]["login"].as_str().unwrap_or("ghost");
        let created = d["createdAt"].as_str().unwrap_or("");
        let body = d["body"].as_str().unwrap_or("").trim().to_string();
        let url = d["url"].as_str().unwrap_or("");

        let mut desc = String::new();
        if !body.is_empty() {
            desc = format!(
                "<pre>{}</pre>",
                escape_html(&body.chars().take(2000).collect::<String>())
            );
        }
        if d["closed"].as_bool().unwrap_or(false) {
            desc.push_str("<br>状态: 已关闭");
        }
        if d["locked"].as_bool().unwrap_or(false) {
            desc.push_str("<br>状态: 已锁定");
        }

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(title.to_string())))
            .link(url.to_string())
            .description(if desc.is_empty() {
                None
            } else {
                Some(desc)
            })
            .pub_date(parse_github_date(created))
            .author(Some(author.to_string()))
            .guid(rss::Guid {
                value: url.to_string(),
                permalink: false,
            })
            .build();
        item_vec.push(item);
    }

    if item_vec.is_empty() {
        return Err(anyhow!(
            "No discussions found for {}/{} (state={})",
            owner,
            repo,
            state
        ));
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Discussions - {}/{}", owner, repo))
        .link(format!("https://github.com/{}/{}/discussions", owner, repo))
        .description(format!("GitHub repository {}/{} discussions", owner, repo))
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