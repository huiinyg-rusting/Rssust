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

///GitHub repository recent Commits via GraphQL API (default branch).
///Params: owner, repo, limit (optional, default 10)
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
        .unwrap_or(10)
        .min(50);

    let token = token()?;
    let query = serde_json::json!({
        "query": r#"query($owner: String!, $repo: String!, $first: Int!) {
            repository(owner: $owner, name: $repo) {
                defaultBranchRef {
                    name
                    target {
                        ... on Commit {
                            history(first: $first) {
                                totalCount
                                nodes {
                                    oid
                                    messageHeadline
                                    committedDate
                                    author { name }
                                }
                            }
                        }
                    }
                }
            }
        }"#,
        "variables": {
            "owner": owner,
            "repo": repo,
            "first": limit,
        }
    }).to_string();

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
            let msg = errors[0]["message"].as_str().unwrap_or("GraphQL 错误");
            return Err(anyhow!("GitHub API error: {}", msg));
        }
    }
    let commits = json["data"]["repository"]["defaultBranchRef"]["target"]["history"]["nodes"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    if commits.is_empty() {
        return Err(anyhow!("Repository not found or no commits. Check owner/repo parameters"));
    }

    let mut item_vec = Vec::new();
    for c in &commits {
        let oid = c["oid"].as_str().unwrap_or("");
        let short = if oid.len() >= 7 { &oid[..7] } else { oid };
        let title = c["messageHeadline"].as_str().unwrap_or("(no message)").to_string();
        let author = c["author"]["name"].as_str().unwrap_or("").to_string();
        let link = format!(
            "https://github.com/{}/{}/commit/{}",
            owner, repo, oid
        );
        let pub_date = c["committedDate"]
            .as_str()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| {
                dt.with_timezone(&Utc)
                    .format("%a, %d %b %Y %H:%M:%S %z")
                    .to_string()
            })
            .unwrap_or_else(now);

        let mut description = format!("commit {}", short);
        if !author.is_empty() {
            description.push_str(&format!("\nAuthor: {}", author));
        }

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(link)
            .description(Some(description))
            .pub_date(pub_date)
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Commits - {}/{}", owner, repo))
        .link(format!("https://github.com/{}/{}", owner, repo))
        .description("GitHub repository recent commits on default branch (GraphQL)")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
