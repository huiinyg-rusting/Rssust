use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let topic = para.get("topic").map(|s| s.as_str()).unwrap_or("trending-news");
    let hub_url = format!("https://apnews.com/hub/{}", topic);

    let html = fetch_reqwest_get_with_headers(
        &hub_url,
        &[
            ("User-Agent", UA),
            ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
            ("Accept-Language", "en-US,en;q=0.9"),
            ("Referer", "https://apnews.com/"),
        ],
    )?;

    let mut links: Vec<String> = Vec::new();
    for m in regex::Regex::new(r#"href="(https://apnews\.com/article/[^"]+)"#)
        .unwrap()
        .captures_iter(&html)
    {
        let url = m[1].to_string();
        if !links.contains(&url) {
            links.push(url);
        }
    }

    if links.is_empty() {
        return Err(anyhow!("在 {} 中找不到文章链接", hub_url));
    }

    let headers = &[
        ("User-Agent", UA),
        ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
        ("Accept-Language", "en-US,en;q=0.9"),
        ("Referer", &hub_url),
    ];

    let mut item_vec = Vec::new();
    for link in &links {
        match fetch_reqwest_get_with_headers(link, headers) {
            Ok(detail_html) => {
                let doc = Html::parse_document(&detail_html);

                let (title, pub_date, author, description) = if let Some(ld) = extract_ldjson(&detail_html) {
                    let t = ld["headline"].as_str().unwrap_or("").to_string();
                    let pd = ld["datePublished"]
                        .as_str()
                        .and_then(|s| {
                            let dt = s.replace('T', " ").replace('Z', "");
                            datetime_str_to_rss(&dt)
                        })
                        .unwrap_or_else(now);

                    let au = ld["author"]
                        .as_array()
                        .and_then(|arr| {
                            arr.iter()
                                .filter_map(|a| a["name"].as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                                .into()
                        })
                        .unwrap_or_default();

                    let mut desc = String::new();
                    if let Some(img) = ld["image"].as_array().and_then(|arr| arr.first()) {
                        if let Some(img_url) = img["url"].as_str() {
                            desc.push_str(&format!("<figure><img src=\"{}\"></figure>", img_url));
                        }
                    }
                    if let Some(d) = ld["description"].as_str() {
                        if !d.is_empty() {
                            desc.push_str(&format!("<p>{}</p>", d));
                        }
                    }
                    if let Some(body) = doc
                        .select(&Selector::parse(".RichTextStoryBody.RichTextBody").unwrap())
                        .next()
                    {
                        desc.push_str(&body.inner_html());
                    }

                    (t, pd, au, desc)
                } else {
                    let t = doc
                        .select(&Selector::parse("title").unwrap())
                        .next()
                        .map(|e| e.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();
                    (t, now(), String::new(), String::new())
                };

                if title.is_empty() {
                    continue;
                }

                let rss_item = ItemBuilder::default()
                    .title(Some(title))
                    .link(Some(link.clone()))
                    .pub_date(pub_date)
                    .author(Some(author))
                    .description(Some(description))
                    .build();
                item_vec.push(rss_item);
            }
            Err(_) => continue,
        }
    }

    let channel = ChannelBuilder::default()
        .title(format!("AP News - {}", topic_name(topic)))
        .link(hub_url)
        .description(format!("AP News {} headlines", topic_name(topic)))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

fn topic_name(topic: &str) -> &str {
    match topic {
        "trending-news" => "Trending News",
        "world" => "World",
        "politics" => "Politics",
        "business" => "Business",
        "technology" => "Technology",
        "science" => "Science",
        "health" => "Health",
        "sports" => "Sports",
        "entertainment" => "Entertainment",
        _ => topic,
    }
}

fn extract_ldjson(html: &str) -> Option<Value> {
    let re = regex::Regex::new(
        r#"<script[^>]*type="application/ld\+json"[^>]*>(.*?)</script>"#,
    )
    .ok()?;
    for cap in re.captures_iter(html) {
        if let Ok(data) = serde_json::from_str::<Value>(&cap[1]) {
            let article = if let Some(arr) = data.as_array() {
                arr.iter().find(|v| v["@type"].as_str() == Some("NewsArticle"))?
            } else {
                &data
            };
            if article["@type"].as_str() == Some("NewsArticle") {
                return Some(article.clone());
            }
        }
    }
    None
}
