use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::{DateTime, Utc};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const GQL: &str = "https://api.github.com/graphql";

fn token() -> Result<String> {
    env_search("GITHUB_TOKEN").ok_or_else(|| {
        anyhow!("Environment variable GITHUB_TOKEN is required (GitHub PAT). See docs.")
    })
}

fn fmt_date(s: &str) -> String {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| {
            dt.with_timezone(&Utc)
                .format("%a, %d %b %Y %H:%M:%S %z")
                .to_string()
        })
        .unwrap_or_else(now)
}

///GitHub Issue / Pull Request comments via GraphQL API.
///Params: owner, repo, limit (optional, default 20, split between issues & PRs, 2 latest comments each)
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
    let per = (limit / 2).max(1);

    let token = token()?;
    let gql = format!(
        r#"{{ repository(owner: \"{}\", name: \"{}\") {{ issues(first: {}) {{ nodes {{ number title url createdAt comments(first: 2) {{ totalCount nodes {{ author {{ login }} body createdAt }} }} }} }} pullRequests(first: {}) {{ nodes {{ number title url createdAt comments(first: 2) {{ totalCount nodes {{ author {{ login }} body createdAt }} }} }} }} }} }}"#,
        owner, repo, per, per
    );
    let query = format!(r#"{{"query":"{}"}}"#, gql);

    let resp = fetch_reqwest_post_json_with_headers(
        GQL,
        &query,
        &[
            ("Authorization", &format!("Bearer {}", token)),
            ("User-Agent", "rssust-github-router/1.0"),
        ],
    )
    .await?;

    let json: Value = serde_json::from_str(&resp)?;
    if let Some(errors) = json["errors"].as_array() {
        if !errors.is_empty() {
            let msg = errors[0]["message"].as_str().unwrap_or("GraphQL error");
            return Err(anyhow!("GitHub API error: {}", msg));
        }
    }

    let mut item_vec = Vec::new();
    let mut push_comments = |nodes: &Value, kind: &str| {
        for node in nodes.as_array().unwrap_or(&vec![]).iter() {
            let number = node["number"].as_i64().unwrap_or(0);
            let title = node["title"].as_str().unwrap_or("").to_string();
            let url = node["url"].as_str().unwrap_or("").to_string();
            let total = node["comments"]["totalCount"].as_i64().unwrap_or(0);
            for c in node["comments"]["nodes"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
            {
                let author = c["author"]["login"].as_str().unwrap_or("unknown").to_string();
                let body = c["body"].as_str().unwrap_or("").to_string();
                let created = c["createdAt"].as_str().unwrap_or("");
                let preview: String = body.chars().take(300).collect::<String>().replace('\n', " ");
                let desc = format!(
                    "{} #{} | comment {} · {} total\nAuthor: {}\n{}",
                    kind, number, fmt_date(created), total, author, preview
                );
                let item = ItemBuilder::default()
                    .title(Some(format!("{} #{}: {}", kind, number, title)))
                    .link(url.clone())
                    .description(Some(desc))
                    .pub_date(fmt_date(created))
                    .build();
                item_vec.push(item);
            }
        }
    };
    push_comments(&json["data"]["repository"]["issues"]["nodes"], "Issue");
    push_comments(&json["data"]["repository"]["pullRequests"]["nodes"], "PR");

    if item_vec.is_empty() {
        return Err(anyhow!("No comments found in this repo's issues/PRs"));
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Issues/PR Comments - {}/{}", owner, repo))
        .link(format!("https://github.com/{}/{}", owner, repo))
        .description("GitHub repository recent Issue / Pull Request comments (GraphQL)")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
