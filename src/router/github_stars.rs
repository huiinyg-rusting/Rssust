use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const GQL: &str = "https://api.github.com/graphql";

fn token() -> Result<String> {
    env_search("GITHUB_TOKEN").ok_or_else(|| {
        anyhow!("Environment variable GITHUB_TOKEN is required (GitHub PAT). See docs.")
    })
}

///GitHub single repository star count via GraphQL API.
///Params: owner, repo
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let owner = para
        .get("owner")
        .cloned()
        .ok_or_else(|| anyhow!("Missing owner parameter (repository owner)"))?;
    let repo = para
        .get("repo")
        .cloned()
        .ok_or_else(|| anyhow!("Missing repo parameter (repository name)"))?;

    let token = token()?;
    let query = format!(
        r#"{{"query":"{{ repository(owner: \"{}\", name: \"{}\") {{ name stargazerCount pushedAt }} }}"}}"#,
        owner, repo
    );

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
    let repo_json = json["data"]["repository"]
        .as_object()
        .ok_or_else(|| anyhow!("Repository not found. Check owner/repo parameters"))?;
    let name = repo_json.get("name").and_then(Value::as_str).unwrap_or("");
    let stars = repo_json.get("stargazerCount").and_then(Value::as_i64).unwrap_or(0);
    let pushed_at = repo_json.get("pushedAt").and_then(Value::as_str).unwrap_or("");

    let description = format!(
        "{} stars\nLast pushed: {}\nGitHub: https://github.com/{}/{}",
        stars, pushed_at, owner, repo
    );

    let item = ItemBuilder::default()
        .title(Some(format!("{} ({})", name, stars)))
        .link(format!("https://github.com/{}/{}", owner, repo))
        .description(Some(description))
        .pub_date(now())
        .build();

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Stars - {}/{}", owner, repo))
        .link(format!("https://github.com/{}/{}", owner, repo))
        .description("GitHub repository star count (GraphQL)")
        .items(vec![item])
        .build();
    Ok(channel.to_string())
}
