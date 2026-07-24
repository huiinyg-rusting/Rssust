use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let tag = para.get("tag").map(|s| s.as_str()).unwrap_or("web_home_page");
    let api_url = format!("https://www.gelonghui.com/api/channels/{}/articles/v8", tag);

    let json_str = fetch_reqwest_get(&api_url)?;
    let json: Value = serde_json::from_str(&json_str)?;
    let result = json["result"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 result 字段"))?;

    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    let mut item_vec = Vec::new();
    for entry in result {
        let article = &entry["data"];
        let title = article["title"].as_str().unwrap_or("");
        let link = article["link"].as_str().unwrap_or("");
        let summary = article["summary"].as_str().unwrap_or("");
        let nick = article["nick"].as_str().unwrap_or("");
        let ts = article["timestamp"].as_i64().unwrap_or(0);
        let source = article["source"].as_str().unwrap_or("");

        if title.is_empty() || link.is_empty() {
            continue;
        }

        let pub_date = timestamp_to_rss(ts);

        let mut description = String::new();
        if !summary.is_empty() {
            description.push_str(&format!("<p>{}</p>", summary));
        }

        if let Ok(detail_html) = fetch_reqwest_get_with_headers(link, &[("User-Agent", ua)]) {
            let doc = Html::parse_document(&detail_html);
            if let Some(article_title) = doc
                .select(&Selector::parse(".article-title").unwrap())
                .next()
            {
                let t = article_title.text().collect::<String>().trim().to_string();
                if !t.is_empty() && description.is_empty() {
                    // just use summary as description
                }
            }
            if let Some(content) = doc
                .select(&Selector::parse("article.article-with-html").unwrap())
                .next()
            {
                let inner = content.inner_html();
                if !inner.is_empty() {
                    description = inner;
                }
            } else if let Some(summary_el) = doc
                .select(&Selector::parse(".article-summary").unwrap())
                .next()
            {
                let s = summary_el.html();
                if !s.is_empty() {
                    if description.is_empty() {
                        description = s;
                    } else {
                        description = format!("{}{}", description, s);
                    }
                }
                if let Some(content) = doc.select(&Selector::parse("article.article-with-html").unwrap()).next() {
                    description = content.inner_html();
                }
            }
        }

        let mut cats = Vec::new();
        if !source.is_empty() {
            cats.push(
                CategoryBuilder::default()
                    .name(source.to_string())
                    .build(),
            );
        }

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(Some(link.to_string()))
            .pub_date(pub_date)
            .author(Some(nick.to_string()))
            .description(Some(description))
            .categories(cats)
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("格隆汇 - {}", tag_name(tag)))
        .link("https://www.gelonghui.com")
        .description("格隆汇财经资讯".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

fn tag_name(tag: &str) -> &str {
    match tag {
        "web_home_page" => "推荐",
        "stock" => "股票",
        "fund" => "基金",
        "new_stock" => "新股",
        "research" => "研报",
        _ => tag,
    }
}
