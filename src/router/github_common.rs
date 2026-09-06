use crate::easyuser::*;
use anyhow::{anyhow, Error, Result};
use serde_json::Value;
use std::collections::HashMap;

///GitHub REST API v3 基地址
pub const API: &str = "https://api.github.com";
///GitHub GraphQL API 端点
pub const GQL: &str = "https://api.github.com/graphql";

///读取 GITHUB_TOKEN（GitHub Personal Access Token）
pub fn token() -> Result<String, Error> {
    env_search("GITHUB_TOKEN").ok_or_else(|| {
        anyhow!("Environment variable GITHUB_TOKEN is required (GitHub PAT). See docs.")
    })
}

///REST API 公共请求头
pub fn rest_headers(token: &str) -> Vec<(&str, String)> {
    vec![
        ("Authorization", format!("Bearer {}", token)),
        ("User-Agent", "rssust-github-router/1.0".to_string()),
        ("Accept", "application/vnd.github+json".to_string()),
    ]
}

///GraphQL API 公共请求头
pub fn gql_headers(token: &str) -> Vec<(&str, String)> {
    vec![
        ("Authorization", format!("Bearer {}", token)),
        ("User-Agent", "rssust-github-router/1.0".to_string()),
    ]
}

fn refs_from<'a>(headers: &'a [(&'a str, String)]) -> Vec<(&'a str, &'a str)> {
    headers.iter().map(|(k, v)| (*k, v.as_str())).collect()
}

///统一解析 GitHub REST 错误对象（`{message, ...}`），无错误返回 Ok
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

///GET 请求 GitHub REST API（自动附带鉴权），返回解析后的 JSON
pub async fn rest_get(url: &str) -> Result<Value, Error> {
    let token = token()?;
    let headers = rest_headers(&token);
    let refs = refs_from(&headers);
    let resp = fetch_reqwest_get_with_headers(url, &refs).await?;
    let json: Value = serde_json::from_str(&resp)?;
    rest_error(&json)?;
    Ok(json)
}

///GET 请求 GitHub REST API，可自定义 Accept 头（如 `application/vnd.github.star+json`）
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

///POST 请求 GitHub GraphQL API（自动附带鉴权），统一检查 errors 后返回 JSON
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

///提取 {owner, repo} 参数
pub fn owner_repo(para: &HashMap<String, String>) -> Result<(String, String), Error> {
    let owner = para
        .get("owner")
        .cloned()
        .ok_or_else(|| anyhow!("Missing owner parameter (repository owner)"))?;
    let repo = para
        .get("repo")
        .cloned()
        .ok_or_else(|| anyhow!("Missing repo parameter (repository name)"))?;
    Ok((owner, repo))
}