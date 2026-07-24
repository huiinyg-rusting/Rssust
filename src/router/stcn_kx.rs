use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let base_url = "https://www.stcn.com";

    let json_str = fetch_reqwest_get_with_headers(
        "https://www.stcn.com/article/list.html?type=kx",
        &[("X-Requested-With", "XMLHttpRequest")],
    )?;

    let json: Value = serde_json::from_str(&json_str)?;
    let list = json["data"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 data 字段"))?;

    let mut item_vec = Vec::new();
    for article in list {
        let title = article["title"].as_str().unwrap_or("");
        let url_path = article["url"].as_str().unwrap_or("");
        let ts_ms = article["time"].as_i64().unwrap_or(0);
        let source = article["source"].as_str().unwrap_or("");
        let content = article["content"].as_str().unwrap_or("");

        if title.is_empty() || url_path.is_empty() {
            continue;
        }

        let link = format!("{}{}", base_url, url_path);
        let pub_date = timestamp_to_rss(ts_ms / 1000);

        let mut description = content.to_string();
        if let Ok(detail_html) = fetch_reqwest_get(&link) {
            let detail_doc = Html::parse_document(&detail_html);
            if let Some(dc) = detail_doc
                .select(&Selector::parse("div.detail-content").unwrap())
                .next()
            {
                let inner = dc.inner_html();
                let clean: String = inner
                    .split('<')
                    .filter(|p| !p.starts_with("script"))
                    .collect::<Vec<_>>()
                    .join("<");
                if !clean.is_empty() {
                    description = clean;
                }
            }
        }

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(Some(link))
            .pub_date(pub_date)
            .author(Some(source.to_string()))
            .description(Some(description))
            .build();

        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("证券时报 - 快讯")
        .link("https://www.stcn.com/article/list/kx.html")
        .description("证券时报网快讯".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
