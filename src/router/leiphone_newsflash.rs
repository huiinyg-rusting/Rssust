use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let json_str = fetch_reqwest_get("https://www.leiphone.com/site/YejieKuaixun")?;
    let json: Value = serde_json::from_str(&json_str)?;
    let articles = json["article"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 article 字段"))?;

    let mut item_vec = Vec::new();
    for article in articles {
        let title = article["title"].as_str().unwrap_or("");
        let url = article["url"].as_str().unwrap_or("");
        let pub_time = article["public_time"].as_str().unwrap_or("");

        if title.is_empty() || url.is_empty() {
            continue;
        }

        let pub_date = if !pub_time.is_empty() {
            chinese_date_to_parse(pub_time).unwrap_or_else(now)
        } else {
            now()
        };

        let mut description = String::new();
        if let Ok(detail_html) = fetch_reqwest_get(url) {
            let doc = Html::parse_document(&detail_html);

            if let Some(img) = doc.select(&Selector::parse(".top-img").unwrap()).next() {
                description.push_str(&img.inner_html());
            }

            if let Some(lead) = doc
                .select(&Selector::parse(".article-lead").unwrap())
                .next()
            {
                description.push_str(&format!(
                    "<blockquote>{}</blockquote>",
                    lead.text().collect::<String>().trim()
                ));
            }

            if let Some(content) = doc
                .select(&Selector::parse(".lph-article-comView").unwrap())
                .next()
            {
                description.push_str(&content.inner_html());
            }
        }

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(Some(url.to_string()))
            .pub_date(pub_date)
            .description(Some(description))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("雷锋网 - 业界资讯")
        .link("https://www.leiphone.com/site/YejieKuaixun")
        .description("雷锋网业界资讯".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
