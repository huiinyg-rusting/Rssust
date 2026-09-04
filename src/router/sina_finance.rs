use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const MAINTAINER: &str = "AI制作 / huiinyg-rusting审核";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

fn truncate(s: &str, n: usize) -> String {
    let mut cs = s.chars();
    let mut out: String = cs.by_ref().take(n).collect();
    if cs.next().is_some() {
        out.push('…');
    }
    out
}

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let count = para
        .get("count")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .clamp(1, 50);
    let url = format!(
        "https://zhibo.sina.com.cn/api/zhibo/feed?page=1&page_size={}&zhibo_id=152",
        count
    );

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &[("User-Agent", UA)])
            .await?
            .as_str(),
    )?;
    let list = json
        .pointer("/result/data/feed/list")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("新浪直播接口缺少 result.data.feed.list"))?;

    let mut item_vec = Vec::new();
    for row in list {
        let rich = row["rich_text"].as_str().unwrap_or("").trim().to_string();
        if rich.is_empty() {
            continue;
        }
        let link = row["docurl"].as_str().unwrap_or("").to_string();
        let create_time = row["create_time"].as_str().unwrap_or("");
        let tags: Vec<String> = row["tag"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|t| t["name"].as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let mut desc = format!("<p>{}</p>", rich);
        if !tags.is_empty() {
            desc.push_str(&format!("<p>标签：{}</p>", tags.join(" / ")));
        }

        item_vec.push(
            ItemBuilder::default()
                .title(Some(truncate(&rich, 60)))
                .link(link.clone())
                .guid(Some(
                    GuidBuilder::default()
                        .value(link.clone())
                        .permalink(true)
                        .build(),
                ))
                .description(Some(desc))
                .pub_date(if create_time.is_empty() {
                    now()
                } else {
                    datetime_str_to_rss(create_time).unwrap_or_else(now)
                })
                .build(),
        );
    }

    let channel = ChannelBuilder::default()
        .title("新浪财经 7×24 快讯")
        .link("https://zhibo.sina.com.cn/finance/live/152")
        .description(format!("新浪财经直播快讯聚合 | {}", MAINTAINER))
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}