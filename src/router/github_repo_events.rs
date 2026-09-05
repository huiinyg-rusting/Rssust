use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::router::github_common::{API, owner_repo, rest_get};
///事件类型 → 过滤 token（与 RSSHub /github/repo_event 的 types 参数一致）
fn event_token(typ: &str) -> &'static str {
    match typ {
        "CreateEvent" => "create",
        "DeleteEvent" => "delete",
        "ForkEvent" => "fork",
        "IssuesEvent" => "issue",
        "IssueCommentEvent" => "issuecomm",
        "MemberEvent" => "member",
        "PullRequestEvent" => "pr",
        "PullRequestReviewCommentEvent" => "prcomm",
        "PullRequestReviewEvent" => "prrev",
        "PublicEvent" => "public",
        "PushEvent" => "push",
        "ReleaseEvent" => "release",
        "WatchEvent" => "star",
        "GollumEvent" => "wiki",
        "CommitCommentEvent" => "cmcomm",
        "DiscussionEvent" => "discussion",
        _ => "",
    }
}

fn event_item(e: &Value, repo_name: &str) -> (String, String) {
    let typ = e["type"].as_str().unwrap_or("");
    let actor = e["actor"]["login"].as_str().unwrap_or("");
    let action = e["payload"]["action"].as_str().unwrap_or("");
    let repo = if repo_name.is_empty() {
        e["repo"]["name"].as_str().unwrap_or("")
    } else {
        repo_name
    };

    let (title, desc): (String, String) = match typ {
        "PushEvent" => {
            let mut desc = String::new();
            if let Some(commits) = e["payload"]["commits"].as_array() {
                for commit in commits {
                    let sha = commit["sha"]
                        .as_str()
                        .unwrap_or("")
                        .chars()
                        .take(7)
                        .collect::<String>();
                    let message = commit["message"]
                        .as_str()
                        .unwrap_or("")
                        .split('\n')
                        .next()
                        .unwrap_or("");
                    desc.push_str(&format!(
                        "<li><a href=\"https://github.com/{}/commit/{}\">{}</a> {}</li>",
                        repo,
                        sha,
                        sha,
                        escape_html(message)
                    ));
                }
            }
            if let Some(branch) = e["payload"]["ref"].as_str() {
                let branch = branch.strip_prefix("refs/heads/").unwrap_or(branch);
                desc.push_str(&format!("<br>分支: {}", escape_html(branch)));
            }
            (
                format!(
                    "{} 推送了 {} 个提交到 {}",
                    actor,
                    e["payload"]["size"].as_i64().unwrap_or(0),
                    repo
                ),
                desc,
            )
        }
        "CreateEvent" | "DeleteEvent" => {
            let verb = if typ == "CreateEvent" { "创建" } else { "删除" };
            let ref_type = e["payload"]["ref_type"].as_str().unwrap_or("");
            let reference = e["payload"]["ref"].as_str().unwrap_or("");
            (
                format!("{} {}了 {} {}", actor, verb, ref_type, reference),
                format!(
                    "类型: {}<br>名称: {}",
                    escape_html(ref_type),
                    escape_html(reference)
                ),
            )
        }
        "ForkEvent" => {
            let fork = e["payload"]["forkee"]["full_name"].as_str().unwrap_or(repo);
            (
                format!("{} Fork 了 {}", actor, fork),
                format!("原仓库: {}", repo),
            )
        }
        "WatchEvent" => {
            let a = if action == "deactivated" {
                "取消关注"
            } else {
                "Star 了"
            };
            (
                format!("{} {} {}", actor, a, repo),
                format!(
                    "动作: {}",
                    escape_html(if action.is_empty() { "started" } else { action })
                ),
            )
        }
        "ReleaseEvent" => {
            let tag = e["payload"]["release"]["tag_name"].as_str().unwrap_or("");
            let html = e["payload"]["release"]["html_url"].as_str().unwrap_or("");
            (
                format!("{} 发布了 {}", actor, tag),
                format!("发布地址: <a href=\"{}\">{}</a>", html, html),
            )
        }
        "IssuesEvent" => {
            let number = e["payload"]["issue"]["number"].as_i64().unwrap_or(0);
            let title = e["payload"]["issue"]["title"].as_str().unwrap_or("").to_string();
            let html = e["payload"]["issue"]["html_url"].as_str().unwrap_or("");
            (
                format!(
                    "{} {}了 Issue #{} {}",
                    actor,
                    if action.is_empty() { "更新" } else { &action },
                    number,
                    &title
                ),
                format!(
                    "<a href=\"{}\">查看 Issue</a><br>状态动作: {}",
                    html,
                    escape_html(if action.is_empty() { "updated" } else { action })
                ),
            )
        }
        "IssueCommentEvent" => {
            let number = e["payload"]["issue"]["number"].as_i64().unwrap_or(0);
            let html = e["payload"]["comment"]["html_url"].as_str().unwrap_or("");
            (
                format!("{} 评论了 Issue #{}", actor, number),
                format!("<a href=\"{}\">查看评论</a>", html),
            )
        }
        "PullRequestEvent" => {
            let number = e["payload"]["pull_request"]["number"].as_i64().unwrap_or(0);
            let pr_title = e["payload"]["pull_request"]["title"].as_str().unwrap_or("").to_string();
            let html = e["payload"]["pull_request"]["html_url"].as_str().unwrap_or("");
            (
                format!(
                    "{} {}了 PR #{} {}",
                    actor,
                    if action.is_empty() { "更新" } else { &action },
                    number,
                    &pr_title
                ),
                format!(
                    "<a href=\"{}\">查看 PR</a><br>状态动作: {}",
                    html,
                    escape_html(if action.is_empty() { "updated" } else { action })
                ),
            )
        }
        "PullRequestReviewCommentEvent" | "PullRequestReviewEvent" => {
            let number = e["payload"]["pull_request"]["number"].as_i64().unwrap_or(0);
            let html = e["payload"]["comment"]["html_url"].as_str().unwrap_or("");
            (
                format!("{} 审查/评论了 PR #{}", actor, number),
                format!("<a href=\"{}\">查看评论</a>", html),
            )
        }
        "PublicEvent" => (
            format!("{} 公开了仓库 {}", actor, repo),
            format!("仓库: {}", repo),
        ),
        "GollumEvent" => {
            let mut pages = Vec::new();
            if let Some(ps) = e["payload"]["pages"].as_array() {
                for p in ps {
                    pages.push(p["page_name"].as_str().unwrap_or("").to_string());
                }
            }
            (
                format!("{} 更新了 Wiki 页面", actor),
                format!("页面: {}", escape_html(&pages.join(", "))),
            )
        }
        "CommitCommentEvent" => {
            let html = e["payload"]["comment"]["html_url"].as_str().unwrap_or("");
            (
                format!("{} 评论了提交", actor),
                format!("<a href=\"{}\">查看评论</a>", html),
            )
        }
        "DiscussionEvent" => {
            let number = e["payload"]["discussion"]["number"].as_i64().unwrap_or(0);
            let dtitle = e["payload"]["discussion"]["title"].as_str().unwrap_or("").to_string();
            let url = e["payload"]["discussion"]["html_url"].as_str().unwrap_or("");
            (
                format!("{} 创建了 Discussion #{} {}", actor, number, &dtitle),
                format!("<a href=\"{}\">查看讨论</a>", url),
            )
        }
        _ => (
            format!("{} 触发了事件 {}", actor, escape_html(typ)),
            String::new(),
        ),
    };
    (no_double_quotes(title), desc)
}

///GitHub 仓库事件流，经 REST `GET /repos/{owner}/{repo}/events` 获取。
///Params: owner, repo, types (可过滤, 逗号分隔, 默认 all), limit (默认30, 最大100)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let (owner, repo) = owner_repo(&para)?;
    let types = para
        .get("types")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "all".to_string());
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(30)
        .min(100);

    let url = format!(
        "{}/repos/{}/{}/events?per_page={}",
        API, owner, repo, limit
    );

    let json: Value = rest_get(&url).await?;
    let events = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    let filter_list: Vec<&str> = if types == "all" {
        Vec::new()
    } else {
        types
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    };

    let mut item_vec = Vec::new();
    for e in events {
        let typ = e["type"].as_str().unwrap_or("");
        if !filter_list.is_empty() && !filter_list.contains(&event_token(typ)) {
            continue;
        }
        let created = e["created_at"].as_str().unwrap_or("");
        let (title, desc) = event_item(e, &format!("{}/{}", owner, repo));
        if title.is_empty() {
            continue;
        }

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(
                e["repo"]["url"]
                    .as_str()
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| format!("https://github.com/{}/{}", owner, repo)),
            )
            .description(if desc.is_empty() { None } else { Some(desc) })
            .pub_date(rfc3339_to_rss(created))
            .author(e["actor"]["login"].as_str().map(String::from))
            .guid(rss::Guid {
                value: format!(
                    "{}@{}",
                    e["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                    created
                ),
                permalink: false,
            })
            .build();
        item_vec.push(item);
    }

    if item_vec.is_empty() {
        return Err(anyhow!(
            "No events found for {}/{} (maybe types filter too strict)",
            owner,
            repo
        ));
    }

    let type_filter = if types == "all" { "All Events" } else { &types };
    let channel = ChannelBuilder::default()
        .title(format!(
            "GitHub Repo Events - {}/{} ({})",
            owner, repo, type_filter
        ))
        .link(format!("https://github.com/{}/{}", owner, repo))
        .description(format!("GitHub events of repository {}/{}", owner, repo))
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