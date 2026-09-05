use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::gql_post;
///GitHub User Followers via GraphQL API.
///Params: username (GitHub login)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let username = para
        .get("username")
        .cloned()
        .ok_or_else(|| anyhow!("Missing username parameter (GitHub login)"))?;

    let query = format!(
        r#"{{"query":"{{ user(login: \"{}\") {{ login name url followers {{ totalCount }} }} }}"}}"#,
        username
    );

    let json: Value = gql_post(&query).await?;
    let user = json["data"]["user"]
        .as_object()
        .ok_or_else(|| anyhow!("User not found. Check username parameter"))?;
    let login = user
        .get("login")
        .and_then(Value::as_str)
        .unwrap_or(&username);
    let name = user.get("name").and_then(Value::as_str).unwrap_or("");
    let followers = user["followers"]["totalCount"].as_i64().unwrap_or(0);
    let fallback_url = format!("https://github.com/{}", username);
    let url = user
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or(&fallback_url);

    let display = if name.is_empty() { login } else { name };
    let description = format!("{} followers: {}\nGitHub: {}", display, followers, url);

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
