use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

const UA: &str = UA_CHROME;

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let topic = para
        .get("topic")
        .map(|s| s.as_str())
        .unwrap_or("trending-news");
    let hub_url = format!("https://apnews.com/hub/{}", topic);

    // AP 已上 Cloudflare 反爬，普通 reqwest 抓不到，改用浏览器指纹伪装抓取；
    // Cloudflare 偶尔回壳页("Just a moment")，检测到就重试
    let mut links: Vec<String> = Vec::new();
    for attempt in 0..6 {
        if let Ok(html) = fetch_browser_get_with_headers(&hub_url, &[
            (
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
            ("Accept-Language", "en-US,en;q=0.9"),
            ("Referer", "https://apnews.com/"),
        ])
        .await
        {
            if !is_cloudflare_shell(&html) {
                for m in
                    regex::Regex::new(r#"href="(https://apnews\.com/article/[^"]+)"#)
                        .unwrap()
                        .captures_iter(&html)
                {
                    let url = m[1].to_string();
                    if !links.contains(&url) {
                        links.push(url);
                    }
                }
                if !links.is_empty() {
                    break;
                }
            }
        }
        if attempt < 5 {
            tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
        }
    }

    if links.is_empty() {
        return Err(anyhow!("在 {} 中找不到文章链接", hub_url));
    }
    // 避免逐篇抓详情耗时过长，最多取 20 条
    links.truncate(20);

    let mut item_vec = Vec::new();
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
    let mut handles = Vec::new();
    for link in links {
        let semaphore = semaphore.clone();
        let ua = UA.to_string();
        let referer = hub_url.clone();
        handles.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            build_article_item(&link, &ua, &referer).await
        }));
    }
    for handle in handles {
        if let Ok(Some(item)) = handle.await {
            item_vec.push(item);
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

///抓单篇详情页并生成 item；失败或撞上 Cloudflare 壳页返回 None
async fn build_article_item(link: &str, ua: &str, referer: &str) -> Option<rss::Item> {
    let headers = [
        ("User-Agent", ua),
        (
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        ),
        ("Accept-Language", "en-US,en;q=0.9"),
        ("Referer", referer),
    ];
    let detail_html = fetch_browser_get_with_headers(link, &headers).await.ok()?;
    if is_cloudflare_shell(&detail_html) {
        return None;
    }

    let doc = Html::parse_document(&detail_html);
    let (title, pub_date, author, description) = if let Some(ld) = extract_ldjson(&detail_html) {
        let t = ld["headline"].as_str().unwrap_or("").to_string();
        let pd = ld["datePublished"]
            .as_str()
            .and_then(|s| {
                let dt = s.replace('T', " ").replace('Z', "");
                utc_str_to_rss(&dt)
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
        if let Some(img) = ld["image"].as_array().and_then(|arr| arr.first())
            && let Some(img_url) = img["url"].as_str()
        {
            desc.push_str(&format!("<figure><img src=\"{}\"></figure>", img_url));
        }
        if let Some(d) = ld["description"].as_str()
            && !d.is_empty()
        {
            desc.push_str(&format!("<p>{}</p>", d));
        }
        if let Some(body) = doc
            .select(&Selector::parse(".RichTextStoryBody.RichTextBody").unwrap())
            .next()
        {
            // 正文里常嵌广告 script/style，剥掉再拼入 description
            desc.push_str(&strip_script_style(&body.inner_html()));
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
        return None;
    }
    Some(
        ItemBuilder::default()
            .title(Some(title))
            .link(Some(link.to_string()))
            .pub_date(pub_date)
            .author(Some(author))
            .description(Some(description))
            .build(),
    )
}

///Cloudflare 反爬壳页识别：体积很小且含挑战特征文案
fn is_cloudflare_shell(html: &str) -> bool {
    (html.len() < 20000 && html.contains("Just a moment")) || html.contains("cf-chl")
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
    let re =
        regex::Regex::new(r#"<script[^>]*type="application/ld\+json"[^>]*>(.*?)</script>"#).ok()?;
    for cap in re.captures_iter(html) {
        if let Ok(data) = serde_json::from_str::<Value>(&cap[1]) {
            let article = if let Some(arr) = data.as_array() {
                arr.iter()
                    .find(|v| v["@type"].as_str() == Some("NewsArticle"))?
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
