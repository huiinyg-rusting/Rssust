use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::{API, rest_get};
///GitHub user recent public events via REST API `GET /users/{username}/events`.
///Params: username, limit (default 30, max 100)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let username = para
        .get("username")
        .cloned()
        .ok_or_else(|| anyhow!("Missing username parameter (GitHub login)"))?;
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(30)
        .min(100);

    let url = format!("{}/users/{}/events?per_page={}", API, username, limit);

    let json: Value = rest_get(&url).await?;
    let events = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    if events.is_empty() {
        return Err(anyhow!("No recent events found. Check username parameter"));
    }

    let mut item_vec = Vec::new();
    for e in events {
        let etype = e["type"].as_str().unwrap_or("");
        let repo_name = e["repo"]["name"].as_str().unwrap_or("");
        let created_at = e["created_at"].as_str().unwrap_or("");
        let id = e["id"].as_str().unwrap_or("");
        let payload = &e["payload"];

        let (title, mut desc, link) = match etype {
            "PushEvent" => {
                let branch = payload["ref"].as_str().unwrap_or("").trim_start_matches("refs/heads/");
                let size = payload["size"].as_i64().unwrap_or(0);
                let mut cmsgs: Vec<String> = Vec::new();
                if let Some(commits) = payload["commits"].as_array() {
                    for c in commits.iter().take(10) {
                        let sha = c["sha"].as_str().unwrap_or("");
                        let short = if sha.len() >= 7 { &sha[..7] } else { sha };
                        let msg = c["message"]
                            .as_str()
                            .unwrap_or("")
                            .lines()
                            .next()
                            .unwrap_or("");
                        cmsgs.push(format!("{} {}", short, msg));
                    }
                }
                let link = format!(
                    "https://github.com/{}/commits/{}",
                    repo_name,
                    if branch.is_empty() { "HEAD" } else { branch }
                );
                (
                    format!("Pushed {} commit(s) to {}", size, repo_name),
                    format!("分支: {}", branch),
                    link,
                )
            }
            "CreateEvent" => {
                let ref_type = payload["ref_type"].as_str().unwrap_or("repository");
                let ref_name = payload["ref"].as_str().unwrap_or("");
                (
                    format!("Created {} in {}", ref_type, repo_name),
                    format!("名称: {}", ref_name),
                    format!("https://github.com/{}", repo_name),
                )
            }
            "DeleteEvent" => {
                let ref_type = payload["ref_type"].as_str().unwrap_or("repository");
                let ref_name = payload["ref"].as_str().unwrap_or("");
                (
                    format!("Deleted {} in {}", ref_type, repo_name),
                    format!("名称: {}", ref_name),
                    format!("https://github.com/{}", repo_name),
                )
            }
            "WatchEvent" => (
                format!("Starred {}", repo_name),
                "Starred the repository".to_string(),
                format!("https://github.com/{}", repo_name),
            ),
            "ForkEvent" => {
                let forkee = payload["forkee"]["full_name"].as_str().unwrap_or("");
                (
                    format!("Forked {} to {}", repo_name, forkee),
                    format!("新仓库: <a href=\"https://github.com/{}\">{}</a>", forkee, forkee),
                    format!("https://github.com/{}", repo_name),
                )
            }
            "IssuesEvent" => {
                let action = payload["action"].as_str().unwrap_or("");
                let number = payload["issue"]["number"].as_i64().unwrap_or(0);
                let ititle = payload["issue"]["title"].as_str().unwrap_or("");
                (
                    format!("{} issue #{} in {}", action, number, repo_name),
                    format!(
                        "<a href=\"https://github.com/{}/issues/{}\">#{} {}</a>",
                        repo_name, number, number, escape_html(ititle)
                    ),
                    format!("https://github.com/{}/issues/{}", repo_name, number),
                )
            }
            "IssueCommentEvent" => {
                let action = payload["action"].as_str().unwrap_or("");
                let number = payload["issue"]["number"].as_i64().unwrap_or(0);
                let body = payload["comment"]["body"].as_str().unwrap_or("");
                (
                    format!("Commented {} on issue #{} in {}", action, number, repo_name),
                    format!(
                        "<a href=\"https://github.com/{}/issues/{}\">issue #{} 评论</a><br><pre>{}</pre>",
                        repo_name,
                        number,
                        number,
                        escape_html(&body.chars().take(500).collect::<String>())
                    ),
                    payload["comment"]["html_url"]
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("https://github.com/{}/issues/{}", repo_name, number)),
                )
            }
            "PullRequestEvent" => {
                let action = payload["action"].as_str().unwrap_or("");
                let number = payload["pull_request"]["number"].as_i64().unwrap_or(0);
                let ptitle = payload["pull_request"]["title"].as_str().unwrap_or("");
                (
                    format!("{} PR #{} in {}", action, number, repo_name),
                    format!(
                        "<a href=\"https://github.com/{}/pull/{}\">PR #{} {}</a>",
                        repo_name,
                        number,
                        number,
                        escape_html(ptitle)
                    ),
                    format!("https://github.com/{}/pull/{}", repo_name, number),
                )
            }
            "PullRequestReviewEvent" => {
                let number = payload["pull_request"]["number"].as_i64().unwrap_or(0);
                (
                    format!("Reviewed PR #{} in {}", number, repo_name),
                    format!("PR: <a href=\"https://github.com/{}/pull/{}\">#{} {}</a>", repo_name, number, number, escape_html(payload["pull_request"]["title"].as_str().unwrap_or(""))),
                    format!("https://github.com/{}/pull/{}", repo_name, number),
                )
            }
            "PullRequestReviewCommentEvent" => {
                let number = payload["pull_request"]["number"].as_i64().unwrap_or(0);
                (
                    format!("Commented on PR #{} in {}", number, repo_name),
                    format!("PR: <a href=\"https://github.com/{}/pull/{}\">#{} {}</a>", repo_name, number, number, escape_html(payload["pull_request"]["title"].as_str().unwrap_or(""))),
                    payload["comment"]["html_url"]
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("https://github.com/{}/pull/{}", repo_name, number)),
                )
            }
            "ReleaseEvent" => {
                let tag = payload["release"]["tag_name"].as_str().unwrap_or("");
                (
                    format!("Released {} in {}", tag, repo_name),
                    format!(
                        "发布: <a href=\"https://github.com/{}/releases/tag/{}\">{}</a>",
                        repo_name, tag, escape_html(tag)
                    ),
                    format!("https://github.com/{}/releases/tag/{}", repo_name, tag),
                )
            }
            "PublicEvent" => (
                format!("Opened {} to public", repo_name),
                "仓库转为公开".to_string(),
                format!("https://github.com/{}", repo_name),
            ),
            "MemberEvent" => {
                let action = payload["action"].as_str().unwrap_or("");
                let member = payload["member"]["login"].as_str().unwrap_or("");
                (
                    format!("{} member {} in {}", action, member, repo_name),
                    format!("成员: {}", member),
                    format!("https://github.com/{}", repo_name),
                )
            }
            "GollumEvent" => {
                let n = payload["pages"].as_array().map(|a| a.len()).unwrap_or(0);
                (
                    format!("Updated wiki page(s) in {}", repo_name),
                    format!("共 {} 个页面变更", n),
                    format!("https://github.com/{}/wiki", repo_name),
                )
            }
            _ => {
                let action = payload["action"].as_str().unwrap_or("");
                (
                    format!("{} {}", etype, repo_name),
                    if action.is_empty() {
                        format!("事件类型: {}", etype)
                    } else {
                        format!("动作: {}", action)
                    },
                    format!("https://github.com/{}", repo_name),
                )
            }
        };

        if !desc.is_empty() {
            desc.push_str(&format!("<br>时间: {}", created_at));
        }

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(link)
                .description(Some(desc))
                .pub_date(rfc3339_to_rss(created_at))
                .guid(rss::Guid {
                    value: id.to_string(),
                    permalink: false,
                })
                .author(Some(username.clone()))
                .build(),
        );
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Events - {}", username))
        .link(format!("https://github.com/{}", username))
        .description(format!("Recent public events of GitHub user {}", username))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}