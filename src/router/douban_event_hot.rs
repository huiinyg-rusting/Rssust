use crate::easyuser::*;
use anyhow::{Error, Result};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let location_id = para.get("locationId").map(|s| s.as_str()).unwrap_or("0");

    let url = format!(
        "https://m.douban.com/rexxar/api/v2/subject_collection/event_hot/items?os=ios&for_mobile=1&callback=&start=0&count=20&loc_id={}",
        location_id
    );

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &[("Referer", "https://m.douban.com/app_topic/event_hot")])?.as_str(),
    )?;

    let items = json
        .pointer("/subject_collection_items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("找不到 subject_collection_items 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    for item in items {
        let title = item["title"].as_str().unwrap_or("");
        let url = item["url"].as_str().unwrap_or("");
        let cover = item["cover"]["url"].as_str().unwrap_or("");
        let subtype = item["subtype"].as_str().unwrap_or("");
        let info = item["info"].as_str().unwrap_or("");
        let price_range = item["price_range"].as_str().unwrap_or("");

        let description = format!(
            "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>{} / {} / {}",
            cover, info, subtype, price_range
        );

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(url.to_string())
            .description(description)
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("豆瓣同城-热门活动-{}", location_id))
        .link("https://m.douban.com/app_topic/event_hot")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
