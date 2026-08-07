use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

///HackerNews via the official Algolia API (news.ycombinator.com 站点对部分服务器不可达，改用官方 API)。
///Params: section (index/newest/ask/show/jobs/best/over 及任意 tags)，type (sources/comments)，value
///  - section=over: 按 points 过滤（value 为最小 points，默认 100）
///  - type=comments: 抓取每篇的评论摘要（前 20 条顶层评论）
///  - value: 追加搜索参数（如 author=xxx）
///限流：请在上游配置 routes.rate_limit["/hackernews"]（建议 30s，遵守 robots Crawl-delay）
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let section = para.get("section").cloned().unwrap_or_else(|| "index".to_string());
    let route_type = para.get("type").cloned().unwrap_or_else(|| "sources".to_string());
    let value = para.get("value").cloned().unwrap_or_default();
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(30)
        .min(50);

    let mut api = String::from("https://hn.algolia.com/api/v1/search?hitsPerPage=");
    api.push_str(&limit.to_string());

    let query = if section == "over" {
        let points = if value.is_empty() { "100".to_string() } else { value.clone() };
        format!("&tags=front_page&numericFilters=points%3E{}", points)
    } else {
        let mut q = String::new();
        if section == "index" {
            q.push_str("&tags=front_page");
        } else {
            let tag = match section.as_str() {
                "ask" => "ask_hn",
                "show" => "show_hn",
                "jobs" => "job",
                "newest" => "story",
                "best" => "story",
                other => other,
            };
            q.push_str(&format!("&tags={}", tag));
        }
        if !value.is_empty() {
            // value 形如 author=xxx 或直接追加
            q.push_str(&format!("&{}", value));
        }
        q
    };
    api.push_str(&query);

    let resp = fetch_reqwest_get_with_headers(&api, &[("User-Agent", "rssust-hn-router/1.0")]).await?;
    let json: Value = serde_json::from_str(&resp)?;
    let hits = json["hits"]
        .as_array()
        .ok_or_else(|| anyhow!("Algolia API 返回异常"))?;

    let root_url = "https://news.ycombinator.com";
    let mut item_vec = Vec::new();
    for hit in hits {
        let id = hit["objectID"].as_str().unwrap_or("");
        let title = hit["title"].as_str().unwrap_or("").to_string();
        let url = hit["url"].as_str().unwrap_or("");
        let author = hit["author"].as_str().unwrap_or("");
        let points = hit["points"].as_i64().unwrap_or(0);
        let num_comments = hit["num_comments"].as_i64().unwrap_or(0);
        let created = hit["created_at"].as_str().unwrap_or("");
        let story_url = format!("{}/item?id={}", root_url, id);

        let link = if route_type == "sources" && !url.is_empty() {
            url.to_string()
        } else {
            story_url.clone()
        };

        let mut desc = format!(
            "<a href=\"{}\">Comments on Hacker News</a> ({} comments, {} points)",
            story_url, num_comments, points
        );
        if !url.is_empty() {
            desc.push_str(&format!(" | <a href=\"{}\">Source</a>", url));
        }

        if route_type == "comments" {
            // 抓取评论摘要
            if let Ok(item_json) = fetch_reqwest_get_with_headers(
                &format!("https://hn.algolia.com/api/v1/items/{}", id),
                &[("User-Agent", "rssust-hn-router/1.0")],
            )
            .await
            {
                if let Ok(iv) = serde_json::from_str::<Value>(&item_json) {
                    let mut comment_html = String::from("<br><br><b>Top comments:</b>");
                    if let Some(kids) = iv["children"].as_array() {
                        let mut count = 0;
                        for k in kids.iter().take(20) {
                            let ca = k["author"].as_str().unwrap_or("");
                            let text = k["text"].as_str().unwrap_or("");
                            comment_html.push_str(&format!(
                                "<div style=\"margin-top:6px\"><b>{}</b>: {}</div>",
                                ca, text
                            ));
                            count += 1;
                        }
                        if count == 0 {
                            comment_html.push_str("<div>No comments yet.</div>");
                        }
                    }
                    desc.push_str(&comment_html);
                }
            }
        }

        let pub_date = chrono::DateTime::parse_from_rfc3339(created)
            .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
            .unwrap_or_else(|_| now());

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(link.clone())
                .description(Some(desc))
                .pub_date(pub_date)
                .guid(if id.is_empty() {
                    rss::Guid {
                        value: link,
                        permalink: false,
                    }
                } else {
                    rss::Guid {
                        value: id.to_string(),
                        permalink: false,
                    }
                })
                .author(if author.is_empty() {
                    None
                } else {
                    Some(author.to_string())
                })
                .build(),
        );
    }

    if item_vec.is_empty() {
        return Err(anyhow!("没有抓取到任何内容"));
    }

    let channel = ChannelBuilder::default()
        .title(format!("Hacker News - {}", section))
        .link(format!("https://news.ycombinator.com/{}", section))
        .description("Hacker News stories (via Algolia API)")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
