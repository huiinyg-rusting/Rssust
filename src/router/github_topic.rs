use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

fn parse_github_date(s: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
        .unwrap_or_else(now)
}

///GitHub Topics page (scraped HTML, no official RSS).
///Params: name (topic name, e.g. framework), qs (optional query string like `l=php&o=desc&s=stars`), limit (default 25)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let name = para
        .get("name")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("Missing name parameter (topic name)"))?;
    let qs = para.get("qs").cloned().unwrap_or_default();
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(25)
        .min(25);

    let mut url = format!("https://github.com/topics/{}", urlencoding::encode(&name));
    if !qs.is_empty() {
        url.push('?');
        url.push_str(&qs);
    }

    let html = fetch_reqwest_get_with_headers(&url, &[("User-Agent", UA)]).await?;
    let doc = Html::parse_document(&html);

    let title_sel = Selector::parse("title").unwrap();
    let page_title = doc
        .select(&title_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| format!("GitHub Topics - {}", name));

    let desc_sel = Selector::parse(".markdown-body").unwrap();
    let channel_desc = doc
        .select(&desc_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let item_sel = Selector::parse("article.border").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let a_sel = Selector::parse("h3 a").unwrap();
    let p_sel = Selector::parse("div > div > p").unwrap();
    let time_sel = Selector::parse("relative-time").unwrap();

    let mut item_vec = Vec::new();
    for art in doc.select(&item_sel) {
        if item_vec.len() >= limit {
            break;
        }

        let Some(h3) = art.select(&h3_sel).next() else {
            continue;
        };
        let full_name = h3.text().collect::<String>().trim().to_string();
        if full_name.is_empty() || !full_name.contains('/') {
            continue;
        }
        let author = full_name.split('/').next().unwrap_or("").to_string();

        let link = art
            .select(&a_sel)
            .last()
            .and_then(|a| a.value().attr("href"))
            .map(|h| {
                if h.starts_with("http") {
                    h.to_string()
                } else {
                    format!("https://github.com{}", h)
                }
            })
            .unwrap_or_else(|| format!("https://github.com/{}", full_name));

        let desc = art
            .select(&p_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let pub_date = art
            .select(&time_sel)
            .next()
            .and_then(|t| t.value().attr("datetime"))
            .map(parse_github_date)
            .unwrap_or_else(now);

        item_vec.push(
            ItemBuilder::default()
                .title(Some(full_name))
                .link(link)
                .description(if desc.is_empty() { None } else { Some(desc) })
                .pub_date(pub_date)
                .author(Some(author))
                .build(),
        );
    }

    if item_vec.is_empty() {
        return Err(anyhow!("No repositories found for this topic"));
    }

    let channel = ChannelBuilder::default()
        .title(page_title)
        .link(url)
        .description(if channel_desc.is_empty() {
            format!("Repositories tagged with GitHub topic {}", name)
        } else {
            channel_desc
        })
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}