use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let html = fetch_reqwest_get("https://news.ifeng.com")?;

    let news_stream_re = Regex::new(r#""newsstream":(\[.*?\]),"cooperation""#)
        .map_err(|_| anyhow!("正则无效"))?;
    let news_json = news_stream_re
        .captures(&html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .ok_or_else(|| anyhow!("找不到 newsstream"))?;

    let stream: Value = serde_json::from_str(news_json)?;
    let items = stream
        .as_array()
        .ok_or_else(|| anyhow!("newsstream 不是数组"))?;

    let mut item_vec = Vec::new();
    for item in items {
        let title = item["title"].as_str().unwrap_or("");
        let link = item["url"].as_str().unwrap_or("");
        let news_time = item["newsTime"].as_str().unwrap_or("");
        let thumbnail = item["thumbnails"]["image"]
            .as_array()
            .and_then(|arr| arr.last())
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if title.is_empty() || link.is_empty() {
            continue;
        }

        let pub_date = datetime_str_to_rss(news_time);

        let description = match fetch_reqwest_get(link) {
            Ok(detail_html) => {
                let mut desc = String::new();
                if !thumbnail.is_empty() {
                    desc.push_str(&format!(
                        r#"<figure><img src="{}"></figure>"#,
                        thumbnail
                    ));
                }

                let content_list_re =
                    Regex::new(r#""contentList":(\[.*?\]),"#)
                        .map_err(|_| anyhow!("正则无效"))?;
                if let Some(caps) = content_list_re.captures(&detail_html) {
                    if let Ok(content_list) =
                        serde_json::from_str::<Value>(caps.get(1).unwrap().as_str())
                    {
                        if let Some(arr) = content_list.as_array() {
                            for entry in arr {
                                let data = entry["data"].as_str().unwrap_or("");
                                if !data.is_empty() {
                                    desc.push_str(data);
                                }
                            }
                        }
                    }
                }

                desc
            }
            Err(_) => String::new(),
        };

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(Some(link.to_string()))
            .pub_date(pub_date)
            .description(Some(description))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("凤凰网资讯")
        .link("https://news.ifeng.com")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
