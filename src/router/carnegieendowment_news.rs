use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::{DateTime, NaiveDateTime, Utc};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const MAINTAINER: &str =
    "huinyg / Carnegie Endowment (international affairs think tank, no official RSS)";

fn parse_carnegie_date(date_str: &str) -> Option<String> {
    let s = date_str.trim();
    // Try formats: "July 22, 2026", "August 5, 2026", "December 2, 2026"
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%B %d, %Y") {
        return Some(
            DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
                .format("%a, %d %b %Y %H:%M:%S %z")
                .to_string(),
        );
    }
    // Try "July 22, 2026 10:00 AM"
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%B %d, %Y %l:%M %p") {
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
        "https://carnegieendowment.org",
        &[("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")],
    ).await?;

    let doc = Html::parse_document(&html);
    let article_sel = Selector::parse("article.flex.min-w-0.flex-1.flex-col.gap-\\[0\\.8rem\\]")
        .map_err(|_| anyhow!("article selector"))?;
    let title_link_sel = Selector::parse("a.min-w-0.flex-1[target='_self'] > span.font-sans")
        .map_err(|_| anyhow!("title selector"))?;
    let link_sel = Selector::parse("a.min-w-0.flex-1[target='_self']")
        .map_err(|_| anyhow!("link selector"))?;
    let date_sel =
        Selector::parse("span:has(svg.text-blue)").map_err(|_| anyhow!("date selector"))?;
    let type_sel =
        Selector::parse("span.px-\\[0\\.8em\\].py-\\[0\\.5em\\].font-mono.text-labelSmall")
            .map_err(|_| anyhow!("type selector"))?;

    let mut item_vec = Vec::new();

    for article in doc.select(&article_sel).take(20) {
        let title = article
            .select(&title_link_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let link = article
            .select(&link_sel)
            .next()
            .and_then(|e| e.value().attr("href"))
            .map(|h| {
                if h.starts_with("http") {
                    h.to_string()
                } else {
                    format!("https://carnegieendowment.org{}", h)
                }
            })
            .unwrap_or_default();

        let content_type = article
            .select(&type_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let date_text = article
            .select(&date_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let pub_date = parse_carnegie_date(&date_text).unwrap_or_else(now);

        if title.is_empty() || link.is_empty() {
            continue;
        }

        let description = if content_type.is_empty() {
            None
        } else {
            Some(format!("[{}] ", content_type))
        };

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(link)
            .pub_date(pub_date)
            .description(description)
            .build();

        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title("Carnegie Endowment - International Peace & Policy")
        .link("https://carnegieendowment.org")
        .description(format!("Carnegie Endowment for International Peace - latest publications, events & videos (no official RSS) | {}", MAINTAINER))
        .items(item_vec)
        .build();

    Ok(channel.to_string())
}
