use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use serde_json::Value;
use std::collections::HashMap;

pub const API: &str = "https://api.github.com";
pub const GQL: &str = "https://api.github.com/graphql";

pub fn token() -> Result<String, Error> {
    env_search("GITHUB_TOKEN").ok_or_else(|| {
        anyhow!("Environment variable GITHUB_TOKEN is required (GitHub PAT). See docs.")
    })
}

pub fn rest_headers(token: &str) -> Vec<(&str, String)> {
    vec![
        ("Authorization", format!("Bearer {}", token)),
        ("User-Agent", "rssust-github-router/1.0".to_string()),
        ("Accept", "application/vnd.github+json".to_string()),
    ]
}

pub fn gql_headers(token: &str) -> Vec<(&str, String)> {
    vec![
        ("Authorization", format!("Bearer {}", token)),
        ("User-Agent", "rssust-github-router/1.0".to_string()),
    ]
}

fn refs_from<'a>(headers: &'a [(&'a str, String)]) -> Vec<(&'a str, &'a str)> {
    headers.iter().map(|(k, v)| (*k, v.as_str())).collect()
}

fn rest_error(json: &Value) -> Result<(), Error> {
    if let Some(msg) = json["message"].as_str() {
        let hint = if json["status"].as_str() == Some("403")
            && (msg.contains("not accessible") || msg.contains("token"))
        {
            " (可能因 PAT 权限不足，请检查 token 的仓库权限)"
        } else {
            ""
        };
        return Err(anyhow!("GitHub API error: {}{}", msg, hint));
    }
    Ok(())
}

pub async fn rest_get(url: &str) -> Result<Value, Error> {
    let token = token()?;
    let headers = rest_headers(&token);
    let refs = refs_from(&headers);
    let resp = fetch_reqwest_get_with_headers(url, &refs).await?;
    let json: Value = serde_json::from_str(&resp)?;
    rest_error(&json)?;
    Ok(json)
}

pub async fn rest_get_with_accept(url: &str, accept: &str) -> Result<Value, Error> {
    let token = token()?;
    let mut headers = rest_headers(&token);
    headers.retain(|(k, _)| *k != "Accept");
    headers.push(("Accept", accept.to_string()));
    let refs = refs_from(&headers);
    let resp = fetch_reqwest_get_with_headers(url, &refs).await?;
    let json: Value = serde_json::from_str(&resp)?;
    rest_error(&json)?;
    Ok(json)
}

pub async fn gql_post(query: &str) -> Result<Value, Error> {
    let token = token()?;
    let headers = gql_headers(&token);
    let refs = refs_from(&headers);
    let resp = fetch_reqwest_post_json_with_headers(GQL, query, &refs).await?;
    let json: Value = serde_json::from_str(&resp)?;
    if let Some(errors) = json["errors"].as_array()
        && !errors.is_empty()
    {
        let msg = errors[0]["message"].as_str().unwrap_or("GraphQL 错误");
        return Err(anyhow!("GitHub API error: {}", msg));
    }
    Ok(json)
}

pub fn owner_repo(para: &HashMap<String, String>) -> Result<(String, String), Error> {
    let owner = crate::easyuser::req_param(para, "owner", "repository owner")?;
    let repo = crate::easyuser::req_param(para, "repo", "repository name")?;
    Ok((owner, repo))
}
