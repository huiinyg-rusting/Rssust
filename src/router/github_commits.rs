use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::{gql_post, owner_repo};
///GitHub repository recent Commits via GraphQL API (default branch).
///Params: owner, repo, limit (optional, default 10)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let (owner, repo) = owner_repo(&para)?;
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10)
        .min(50);

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
    })
    .to_string();

    let json: Value = gql_post(&query).await?;
    let commits = json["data"]["repository"]["defaultBranchRef"]["target"]["history"]["nodes"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    if commits.is_empty() {
        return Err(anyhow!(
            "Repository not found or no commits. Check owner/repo parameters"
        ));
    }

    let mut item_vec = Vec::new();
    for c in &commits {
        let oid = c["oid"].as_str().unwrap_or("");
        let short = if oid.len() >= 7 { &oid[..7] } else { oid };
        let title = c["messageHeadline"]
            .as_str()
            .unwrap_or("(no message)")
            .to_string();
        let author = c["author"]["name"].as_str().unwrap_or("").to_string();
        let link = format!("https://github.com/{}/{}/commit/{}", owner, repo, oid);
        let pub_date = rfc3339_to_rss_utc(c["committedDate"].as_str().unwrap_or(""));

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
