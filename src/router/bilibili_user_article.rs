use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let uid = para
        .get("uid")
        .ok_or_else(|| anyhow!("缺少 uid 参数"))?;

    let url = format!(
        "https://api.bilibili.com/x/polymer/web-dynamic/v1/opus/feed/space?host_mid={}",
        uid
    );
    let referer = format!("https://space.bilibili.com/{}/article", uid);
    let headers: Vec<(&str, &str)> = vec![("Referer", referer.as_str())];

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &headers)?.as_str(),
    )?;

    let data = json
        .pointer("/data/items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data/items 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    let mut author = String::from("UP主");

    for item in data {
        let content = item["content"].as_str().unwrap_or_default();
        let jump_url = item["jump_url"].as_str().unwrap_or_default();
        let link = if jump_url.starts_with("http") {
            jump_url.to_string()
        } else {
            format!("https:{}", jump_url)
        };

        if let Some(name) = item
            .pointer("/author/name")
            .and_then(Value::as_str)
        {
            if author == "UP主" {
                author = name.to_string();
            }
        }

        // 获取详情页解析描述
        let detail_html = fetch_reqwest_get_with_headers(
            &link,
            &[
                ("Referer", referer.as_str()),
                ("User-Agent",
                 "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
            ],
        )
        .unwrap_or_default();

        let description = if !detail_html.is_empty() {
            let document = Html::parse_document(&detail_html);
            let selector = Selector::parse("div.opus-module-content").unwrap();
            document
                .select(&selector)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_else(|| content.to_string())
        } else {
            content.to_string()
        };

        let pub_date = now();

        let rss_item = ItemBuilder::default()
            .title(Some(no_double_quotes(item["content"].to_string())))
            .link(link)
            .description(description)
            .pub_date(pub_date)
            .author(author.clone())
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("{} 的 bilibili 图文", author))
        .link(format!("https://space.bilibili.com/{}/article", uid))
        .description(format!("{} 的 bilibili 图文", author))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
