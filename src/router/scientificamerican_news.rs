use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::{DateTime, Utc};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const MAINTAINER: &str = "huinyg / Scientific American (science & technology, no official RSS)";

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let html = fetch_reqwest_get_with_headers(
        "https://www.scientificamerican.com",
        &[("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")],
    ).await?;

    let doc = Html::parse_document(&html);
    let card_sel = Selector::parse("article[data-click-position^='feed-card']")
        .map_err(|_| anyhow!("card selector"))?;
    let title_sel =
        Selector::parse("a[class*='cardLink']").map_err(|_| anyhow!("title selector"))?;
    let dek_sel = Selector::parse("div[class^='dek-'] p").map_err(|_| anyhow!("dek selector"))?;
    let time_sel = Selector::parse("time").map_err(|_| anyhow!("time selector"))?;
    let author_sel =
        Selector::parse("span[class^='authors-']").map_err(|_| anyhow!("author selector"))?;

    let mut item_vec = Vec::new();

    for card in doc.select(&card_sel).take(20) {
        let title = card
            .select(&title_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let link = card
            .select(&title_sel)
            .next()
            .and_then(|e| e.value().attr("href"))
            .map(|h| {
                if h.starts_with("http") {
                    h.to_string()
                } else {
                    format!("https://www.scientificamerican.com{}", h)
                }
            })
            .unwrap_or_default();

        let description = card
            .select(&dek_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let author = card
            .select(&author_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let pub_date = card
            .select(&time_sel)
            .next()
            .and_then(|e| e.value().attr("datetime"))
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| {
                dt.with_timezone(&Utc)
                    .format("%a, %d %b %Y %H:%M:%S %z")
                    .to_string()
            })
            .unwrap_or_else(now);

        if title.is_empty() || link.is_empty() {
            continue;
        }

        let mut desc = String::new();
        if !author.is_empty() {
            desc.push_str(&format!("By {}<br/>", author));
        }
        if !description.is_empty() {
            desc.push_str(&description);
        }

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(link)
            .pub_date(pub_date)
            .description(if desc.is_empty() { None } else { Some(desc) })
            .build();

        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title("Scientific American - Science & Technology News")
        .link("https://www.scientificamerican.com")
        .description(format!(
            "Scientific American latest articles - the essential guide to the most awe-inspiring advances in science & technology (no official RSS) | {}",
            MAINTAINER
        ))
        .items(item_vec)
        .build();

    Ok(channel.to_string())
}
