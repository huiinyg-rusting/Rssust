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

///GitHub User Followers via GraphQL API.
///Params: username (GitHub login)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let username = para
        .get("username")
        .cloned()
        .ok_or_else(|| anyhow!("Missing username parameter (GitHub login)"))?;

    let token = token()?;
    let query = format!(
        r#"{{"query":"{{ user(login: \"{}\") {{ login name url followers {{ totalCount }} }} }}"}}"#,
        username
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
    let user = json["data"]["user"]
        .as_object()
        .ok_or_else(|| anyhow!("User not found. Check username parameter"))?;
    let login = user.get("login").and_then(Value::as_str).unwrap_or(&username);
    let name = user.get("name").and_then(Value::as_str).unwrap_or("");
    let followers = user["followers"]["totalCount"].as_i64().unwrap_or(0);
    let fallback_url = format!("https://github.com/{}", username);
    let url = user
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or(&fallback_url);

    let display = if name.is_empty() { login } else { name };
    let description = format!(
        "{} followers: {}\nGitHub: {}",
        display, followers, url
    );

    let item = ItemBuilder::default()
        .title(Some(format!("{} ({} followers)", display, followers)))
        .link(format!("https://github.com/{}?tab=followers", username))
        .description(Some(description))
        .pub_date(now())
        .build();

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Followers - {}", login))
        .link(format!("https://github.com/{}", username))
        .description("GitHub user followers via GraphQL API")
        .items(vec![item])
        .build();
    Ok(channel.to_string())
}
