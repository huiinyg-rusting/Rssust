use crate::easyuser::*;
use anyhow::{Error, Result};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let url = "https://gw.m.163.com/nc/api/v1/feed/static/normal-list?start=0&tid=T1573700340788&size=30";

    let json: Value = serde_json::from_str(fetch_reqwest_get(url)?.as_str())?;

    let list = json
        .pointer("/data/items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("找不到 data/items 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    for item in list {
        let title = item["title"].as_str().unwrap_or("");
        let author = item["source"].as_str().unwrap_or("");
        let ptime = item["ptime"].as_str().unwrap_or("");
        let digest = item["digest"].as_str().unwrap_or("");
        let imgsrc = item["imgsrc"].as_str().unwrap_or("");
        let docid = item["docid"].as_str().unwrap_or("");
        let default_link = format!("https://c.m.163.com/news/a/{}.html", docid);
        let link = item["url"].as_str().unwrap_or(&default_link);

        let description = format!("<p>{}</p><img src=\"{}\" referrerpolicy=\"no-referrer\">", digest, imgsrc);

        let pub_date = if !ptime.is_empty() {
            datetime_str_to_rss(ptime).unwrap_or_else(now)
        } else {
            now()
        };

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(link.to_string())
            .description(description)
            .author(Some(author.to_string()))
            .pub_date(pub_date)
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("今日关注 - 网易新闻".to_string())
        .link("https://wp.m.163.com/163/html/newsapp/todayFocus/index.html".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
