use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::{DateTime, NaiveDateTime, Utc};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const MAINTAINER: &str = "huinyg / Smithsonian Magazine (official RSS returns 403)";

fn parse_smithsonian_date(date_str: &str) -> Option<String> {
    let s = date_str.trim().replace("p.m.", "PM").replace("a.m.", "AM");
    // Try formats:
    // "July 27, 2026"
    // "July 31, 2026 6:33 PM"
    if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%B %d, %Y %l:%M %p") {
        return Some(
            DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)
                .format("%a, %d %b %Y %H:%M:%S %z")
                .to_string(),
        );
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%B %d, %Y") {
        return Some(
            DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
                .format("%a, %d %b %Y %H:%M:%S %z")
                .to_string(),
        );
    }
    None
}

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let html = fetch_reqwest_get_with_headers(
        "https://www.smithsonianmag.com",
        &[("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")],
    ).await?;

    let doc = Html::parse_document(&html);
    let item_sel =
        Selector::parse("div.article-list-item").map_err(|_| anyhow!("item selector"))?;
    let title_sel = Selector::parse("h3 > a").map_err(|_| anyhow!("title selector"))?;
    let summary_sel = Selector::parse("p.summary").map_err(|_| anyhow!("summary selector"))?;
    let time_sel = Selector::parse("time.pub-date").map_err(|_| anyhow!("time selector"))?;
    let link_sel = Selector::parse("h3 > a").map_err(|_| anyhow!("link selector"))?;

    let mut item_vec = Vec::new();

    for item in doc.select(&item_sel).take(20) {
        let title = item
            .select(&title_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let link = item
            .select(&link_sel)
            .next()
            .and_then(|e| e.value().attr("href"))
            .map(|h| {
                if h.starts_with("http") {
                    h.to_string()
                } else {
                    format!("https://www.smithsonianmag.com{}", h)
                }
            })
            .unwrap_or_default();

        let description = item
            .select(&summary_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let pub_date = item
            .select(&time_sel)
            .next()
            .and_then(|e| {
                e.text()
                    .collect::<String>()
                    .trim()
                    .to_string()
                    .parse::<String>()
                    .ok()
            })
            .and_then(|s| parse_smithsonian_date(&s))
            .unwrap_or_else(now);

        if title.is_empty() || link.is_empty() {
            continue;
        }

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(link)
            .pub_date(pub_date)
            .description(if description.is_empty() {
                None
            } else {
                Some(description)
            })
            .build();

        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title("Smithsonian Magazine - Latest Science, History & Culture")
        .link("https://www.smithsonianmag.com")
        .description(format!(
            "Smithsonian Magazine latest articles (official RSS returns 403) | {}",
            MAINTAINER
        ))
        .items(item_vec)
        .build();

    Ok(channel.to_string())
}
