use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let api_url = "https://app.eeo.com.cn?app=article&controller=index&action=getMoreArticle&catid=3690&uuid=b048c7211db949eeb7443cd5b9b3bfe3&page=1&pageSize=50";
    let json_str = fetch_reqwest_get(api_url).await?;
    let json: Value = serde_json::from_str(&json_str)?;
    let data = json["data"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 data 字段"))?;

    let mut item_vec = Vec::new();
    for item in data {
        let title = item["title"].as_str().unwrap_or("");
        let link = item["url"].as_str().unwrap_or("");
        let pub_time = item["published"].as_str().unwrap_or("");
        let intro = item["description"].as_str().unwrap_or("");

        if title.is_empty() || link.is_empty() {
            continue;
        }

        let pub_date = if !pub_time.is_empty() {
            datetime_str_to_rss(pub_time).unwrap_or_else(now)
        } else {
            now()
        };

        let mut description = String::new();
        if let Ok(detail_html) = fetch_reqwest_get(link).await {
            let doc = Html::parse_document(&detail_html);

            let sel_h1 = Selector::parse("h1").unwrap();
            let sel_content = Selector::parse("div.xx_boxsing").unwrap();

            if let Some(content_div) = doc.select(&sel_content).next() {
                if !intro.is_empty() {
                    description.push_str(&format!("<blockquote>{}</blockquote>", intro));
                }
                description.push_str(&content_div.inner_html());
            } else if let Some(h1) = doc.select(&sel_h1).next() {
                if !intro.is_empty() {
                    description.push_str(&format!("<blockquote>{}</blockquote>", intro));
                }
                description.push_str(&h1.inner_html());
            }
        }

        if description.is_empty() {
            description = intro.to_string();
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
        .title("经济观察报 - 快讯")
        .link("https://www.eeo.com.cn/kuaixun/")
        .description("经济观察报快讯".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
