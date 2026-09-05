use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::{gql_post, owner_repo};
///GitHub single repository star count via GraphQL API.
///Params: owner, repo
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let (owner, repo) = owner_repo(&para)?;

    let query = format!(
        r#"{{"query":"{{ repository(owner: \"{}\", name: \"{}\") {{ name stargazerCount pushedAt }} }}"}}"#,
        owner, repo
    );

    let json: Value = gql_post(&query).await?;
    let repo_json = json["data"]["repository"]
        .as_object()
        .ok_or_else(|| anyhow!("Repository not found. Check owner/repo parameters"))?;
    let name = repo_json.get("name").and_then(Value::as_str).unwrap_or("");
    let stars = repo_json
        .get("stargazerCount")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let pushed_at = repo_json
        .get("pushedAt")
        .and_then(Value::as_str)
        .unwrap_or("");

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
