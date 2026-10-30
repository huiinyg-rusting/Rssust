use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

/// lobste.rs 热门/最新文章（对应 RSSHub /lobste.rs 系列）。
/// 上游：https://lobste.rs 公开 JSON 接口：
///  - 热门：https://lobste.rs/hottest.json
///  - 最新：https://lobste.rs/newest.json
///  - 标签：https://lobste.rs/t/{tag}.json（仅热门排序）
/// 参数：
///  - sort：hot（默认）/ newest
///  - tag：可选，只取某标签，如 rust
///  - page：可选，分页页码（从 1 开始，默认 1，上游每页约 25 条）
///
/// 限流：建议配置 routes.rate_limit["/lobsters_hot"] = 600（遵守 robots Crawl-delay）
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let sort = para
        .get("sort")
        .cloned()
        .unwrap_or_else(|| "hot".to_string());
    let tag = para.get("tag").cloned().unwrap_or_default();
    let page = opt_usize(&para, "page", 1).max(1);

    let url = if !tag.is_empty() {
        format!(
            "https://lobste.rs/t/{}.json?page={}",
            urlencoding::encode(&tag),
            page
        )
    } else {
        let path = match sort.as_str() {
            "newest" | "new" => "newest",
            _ => "hottest",
        };
        format!("https://lobste.rs/{}.json?page={}", path, page)
    };

    let resp = fetch_reqwest_get(&url).await?;
    let list: Value = serde_json::from_str(&resp)
        .map_err(|e| anyhow!("lobste.rs 返回非 JSON：{}（{}）", e, truncate(&resp, 120)))?;
    let stories = list
        .as_array()
        .ok_or_else(|| anyhow!("lobste.rs 返回异常，预期数组"))?;

    let mut item_vec = Vec::new();
    for story in stories {
        let short_id = story["short_id"].as_str().unwrap_or("");
        let title = story["title"].as_str().unwrap_or("").to_string();
        let article_url = story["url"].as_str().unwrap_or("");
        let comments_url = story["comments_url"].as_str().unwrap_or("");
        let short_id_url = story["short_id_url"].as_str().unwrap_or("");
        let submitter = story["submitter_user"]["username"]
            .as_str()
            .or_else(|| story["submitter_user"].as_str())
            .unwrap_or("");
        let score = story["score"].as_i64().unwrap_or(0);
        let comments = story["comment_count"].as_i64().unwrap_or(0);
        let created = story["created_at"].as_str().unwrap_or("");
        let tags: Vec<String> = story["tags"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|t| t.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let summary = story["description_plain"].as_str().unwrap_or("").trim();

        /* 无外链的“讨论帖”退回讨论页本身 */
        let link = if article_url.is_empty() {
            if comments_url.is_empty() {
                short_id_url.to_string()
            } else {
                comments_url.to_string()
            }
        } else {
            article_url.to_string()
        };

        /* 描述：作者 + 分数 + 评论数 + 标签 + 摘要 */
        let mut desc = String::new();
        if !submitter.is_empty() {
            desc.push_str(&format!("by <b>{}</b>", submitter));
        }
        desc.push_str(&format!(
            " | {} points | <a href=\"{}\">{} comments on lobste.rs</a>",
            score, comments_url, comments
        ));
        if !tags.is_empty() {
            desc.push_str(&format!("<br>tags: {}", tags.join(", ")));
        }
        if !summary.is_empty() {
            desc.push_str(&format!("<br><br>{}", strip_script_style(summary)));
        }

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(Some(link.clone()))
                .description(Some(desc))
                .pub_date(rfc3339_to_rss(created))
                .guid(if short_id.is_empty() {
                    rss::Guid {
                        value: link,
                        permalink: false,
                    }
                } else {
                    rss::Guid {
                        value: short_id.to_string(),
                        permalink: false,
                    }
                })
                .build(),
        );
    }

    if item_vec.is_empty() {
        return Err(anyhow!("没有抓取到任何内容"));
    }

    let channel_title = if !tag.is_empty() {
        format!("lobste.rs - {} (hot)", tag)
    } else {
        format!("lobste.rs - {}", sort)
    };
    let channel = ChannelBuilder::default()
        .title(channel_title)
        .link("https://lobste.rs/")
        .description("lobste.rs stories")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
