use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const MAINTAINER: &str = "AI制作 / huiinyg-rusting审核";
const UA: &str = UA_CHROME;
const URL: &str = "https://www.toutiao.com/hot-event/hot-board/?origin=toutiao_pc";

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let count = para
        .get("count")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(50)
        .clamp(1, 50);

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(URL, &[("User-Agent", UA)])
            .await?
            .as_str(),
    )?;
    let arr = json["data"]
        .as_array()
        .ok_or_else(|| anyhow!("今日头条热榜返回缺少 data"))?;

    let mut item_vec = Vec::new();
    for it in arr.iter().take(count) {
        let title = it["Title"].as_str().unwrap_or("").to_string();
        if title.is_empty() {
            continue;
        }
        let link = it["Url"].as_str().unwrap_or("").to_string();
        let hot = it["HotValue"].as_str().unwrap_or("");
        let label = it["Label"].as_str().unwrap_or("");
        let category = it["InterestCategory"].as_str().unwrap_or("");

        let mut desc = String::new();
        if !hot.is_empty() {
            desc.push_str(&format!("热度值：{}", hot));
        }
        if !label.is_empty() {
            desc.push_str(&format!(" · 标签：{}", label));
        }
        if !category.is_empty() {
            desc.push_str(&format!(" · 分类：{}", category));
        }
        if desc.is_empty() {
            desc.push_str("今日头条热搜");
        }

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(link.clone())
                .guid(Some(
                    GuidBuilder::default()
                        .value(link.clone())
                        .permalink(true)
                        .build(),
                ))
                .description(Some(desc))
                .pub_date(now())
                .build(),
        );
    }

    let channel = ChannelBuilder::default()
        .title("今日头条热榜")
        .link(URL.to_string())
        .description(format!("今日头条热搜榜单 | {}", MAINTAINER))
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}