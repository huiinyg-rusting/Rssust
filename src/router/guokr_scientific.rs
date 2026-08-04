use crate::easyuser::*;
use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

///果壳网科学人：抓取 science_api 列表，抓详情页补全正文。
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(50);

    let resp = fetch_reqwest_get_with_headers(
        "https://www.guokr.com/beta/proxy/science_api/articles?retrieve_type=by_category&page=1",
        &[("User-Agent", UA)],
    )
    .await?;
    let list: Vec<Value> = serde_json::from_str(&resp)?;

    let mut item_vec = Vec::new();
    for item in list.iter().take(limit) {
        let id = item["id"].as_i64().unwrap_or(0);
        let title = item["title"].as_str().unwrap_or("").to_string();
        let summary = item["summary"].as_str().unwrap_or("").to_string();
        let link = format!("https://www.guokr.com/article/{}/", id);
        let author = item["author"]["nickname"].as_str().unwrap_or("").to_string();

        let pub_date = item["date_published"]
            .as_str()
            .and_then(|s| {
                DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| {
                        dt.with_timezone(&Utc)
                            .format("%a, %d %b %Y %H:%M:%S %z")
                            .to_string()
                    })
            })
            .unwrap_or_else(now);

        let mut description = summary.clone();
        if let Ok(detail) = fetch_reqwest_get(&format!(
            "https://apis.guokr.com/minisite/article/{}.json",
            id
        ))
        .await
        {
            if let Ok(detail_json) = serde_json::from_str::<Value>(&detail) {
                if let Some(content) = detail_json["result"]["content"].as_str() {
                    if !content.is_empty() {
                        description = content.to_string();
                    }
                }
            }
        }

        let rss_item = ItemBuilder::default()
            .title(Some(title))
            .link(link)
            .description(Some(description))
            .pub_date(pub_date)
            .author(Some(author))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("果壳网 科学人")
        .link("https://www.guokr.com/scientific")
        .description("果壳网 科学人")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
