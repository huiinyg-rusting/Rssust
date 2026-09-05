use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::{gql_post, owner_repo};
///GitHub Issue / Pull Request comments via GraphQL API.
///Params: owner, repo, limit (optional, default 20, split between issues & PRs, 2 latest comments each)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let (owner, repo) = owner_repo(&para)?;
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(50);
    let per = (limit / 2).max(1);

    let gql = format!(
        r#"{{ repository(owner: \"{}\", name: \"{}\") {{ issues(first: {}) {{ nodes {{ number title url createdAt comments(first: 2) {{ totalCount nodes {{ author {{ login }} body createdAt }} }} }} }} pullRequests(first: {}) {{ nodes {{ number title url createdAt comments(first: 2) {{ totalCount nodes {{ author {{ login }} body createdAt }} }} }} }} }} }}"#,
        owner, repo, per, per
    );
    let query = format!(r#"{{"query":"{}"}}"#, gql);

    let json: Value = gql_post(&query).await?;

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
                let author = c["author"]["login"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                let body = c["body"].as_str().unwrap_or("").to_string();
                let created = c["createdAt"].as_str().unwrap_or("");
                let preview: String = body
                    .chars()
                    .take(300)
                    .collect::<String>()
                    .replace('\n', " ");
                let desc = format!(
                    "{} #{} | comment {} · {} total\nAuthor: {}\n{}",
                    kind,
                    number,
                    rfc3339_to_rss_utc(created),
                    total,
                    author,
                    preview
                );
                let item = ItemBuilder::default()
                    .title(Some(format!("{} #{}: {}", kind, number, title)))
                    .link(url.clone())
                    .description(Some(desc))
                    .pub_date(rfc3339_to_rss_utc(created))
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
