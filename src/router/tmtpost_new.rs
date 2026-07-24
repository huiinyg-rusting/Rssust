use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let list_json = fetch_reqwest_get_with_headers(
        "https://api.tmtpost.com/v1/lists/new?limit=30",
        &[("app-version", "web1.0")],
    )?;

    let list: Value = serde_json::from_str(&list_json)?;
    let items = list["data"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 data 字段"))?;

    let mut item_vec = Vec::new();
    for item in items {
        let title = item["title"].as_str().unwrap_or("");
        let guid = item["guid"].as_str().unwrap_or("");
        let link = item["share_link"].as_str().unwrap_or("");
        let ts = item["time_published"].as_i64().unwrap_or(0);

        if title.is_empty() || guid.is_empty() {
            continue;
        }

        let pub_date = timestamp_to_rss(ts);
        let detail_url = format!(
            "https://api.tmtpost.com/v1/posts/{}?fields=authors;tags;categories;featured_image;share_link",
            guid
        );

        let mut description = String::new();

        if let Some(summary) = item["summary"].as_str() {
            if !summary.is_empty() {
                description.push_str(&format!("<p><strong>{}</strong></p>", summary));
            }
        }

        if let Ok(detail_json) = fetch_reqwest_get_with_headers(&detail_url, &[("app-version", "web1.0")])
        {
            if let Ok(detail) = serde_json::from_str::<Value>(&detail_json) {
                if let Some(data) = detail["data"].as_object() {
                    if let Some(main_html) = data["main"].as_str() {
                        if !main_html.is_empty() {
                            description = main_html.to_string();
                        }
                    }
                }
            }
        }

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(Some(link.to_string()))
            .pub_date(pub_date)
            .description(Some(description))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("钛媒体 - 最新")
        .link("https://www.tmtpost.com")
        .description("钛媒体最新文章".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
