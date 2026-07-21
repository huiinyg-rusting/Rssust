use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let period = para.get("period").map(|s| s.as_str()).unwrap_or("day");

    let json: Value = serde_json::from_str(
        fetch_reqwest_get("https://api-one-wscn.awtmt.com/apiv1/content/articles/hot?period=all")?
            .as_str(),
    )?;

    let period_key = format!("{}_items", period);
    let items = json
        .pointer("/data")
        .and_then(|d| d.get(&period_key))
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data.{}_items", period))?;

    let mut item_vec = Vec::new();
    for item in items {
        let title = item["title"].as_str().unwrap_or("");
        let link = item["uri"].as_str().unwrap_or("");
        let pub_ts = item["display_time"].as_i64().unwrap_or(0);
        let pub_date = timestamp_to_rss(pub_ts);
        let id = item["id"].as_i64().unwrap_or(0);
        let description = format!(
            "阅读量: {} | 评论: {}<br><a href=\"{}\">查看原文</a>",
            item["pageviews"].as_i64().unwrap_or(0),
            item["comment_count"].as_i64().unwrap_or(0),
            link
        );

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(if link.is_empty() {
                format!("https://wallstreetcn.com/articles/{}", id)
            } else {
                link.to_string()
            })
            .pub_date(pub_date)
            .description(Some(description))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!(
            "华尔街见闻 - 最热文章 ({})",
            if period == "day" { "当日" } else { "当周" }
        ))
        .link("https://wallstreetcn.com")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
