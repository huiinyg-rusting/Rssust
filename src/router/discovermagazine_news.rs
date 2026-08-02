use crate::easyuser::*;
use anyhow::{Error, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const MAINTAINER: &str = "huinyg / Discover Magazine (science news, no official RSS)";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let html =
        fetch_reqwest_get_with_headers("https://www.discovermagazine.com", &[("User-Agent", UA)])
            .await?;

    // Extract cards first (synchronous, doc dropped here so it can be reused across awaits).
    let cards = extract_cards(&html);

    let mut items = Vec::new();
    for (title, link, summary, category) in cards {
        let pub_date = fetch_article_date(&link).await;

        let mut desc = String::new();
        if !category.is_empty() {
            desc.push_str(&format!("[{}]<br/>", category));
        }
        if !summary.is_empty() {
            desc.push_str(&summary);
        }

        items.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(link)
                .pub_date(pub_date)
                .description(if desc.is_empty() { None } else { Some(desc) })
                .build(),
        );
    }

    let channel = ChannelBuilder::default()
        .title("Discover Magazine - Science News")
        .link("https://www.discovermagazine.com")
        .description(format!(
            "Discover Magazine latest science news (no official RSS) | {}",
            MAINTAINER
        ))
        .items(items)
        .build();

    Ok(channel.to_string())
}

fn extract_cards(html: &str) -> Vec<(String, String, String, String)> {
    let doc = Html::parse_document(html);
    let item_sel = Selector::parse("a[href]").unwrap();
    let summary_sel = Selector::parse("span[data-summary]").unwrap();
    let cat_sel = Selector::parse("p.category-label a").unwrap();

    let mut seen: Vec<String> = Vec::new();
    let mut cards: Vec<(String, String, String, String)> = Vec::new();

    for node in doc.select(&item_sel) {
        let href = node.value().attr("href").unwrap_or_default().to_string();
        if !href.starts_with("/") {
            continue;
        }
        // article links look like /slug-49475
        let is_article = href
            .rsplit('-')
            .next()
            .map(|tail| !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or(false);
        if !is_article {
            continue;
        }

        let title = node.text().collect::<String>().trim().to_string();
        if title.is_empty() {
            continue;
        }

        let full_link = format!("https://www.discovermagazine.com{}", href);

        if seen.contains(&full_link) {
            continue;
        }
        seen.push(full_link.clone());

        // summary & category live in an ancestor card container of the <a> element;
        // climb up a few levels until the card container (identified by category) is found
        let mut container = node.parent().and_then(scraper::ElementRef::wrap);
        let mut summary = String::new();
        let mut category = String::new();
        for _ in 0..3 {
            let current = match container {
                Some(c) => c,
                None => break,
            };
            if summary.is_empty() {
                summary = current
                    .select(&summary_sel)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
            }
            if category.is_empty() {
                category = current
                    .select(&cat_sel)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
            }
            if !category.is_empty() {
                break;
            }
            container = current.parent().and_then(scraper::ElementRef::wrap);
        }

        cards.push((title, full_link, summary, category));

        if cards.len() >= 20 {
            break;
        }
    }

    cards
}

async fn fetch_article_date(link: &str) -> String {
    if let Ok(html) = fetch_reqwest_get_with_headers(link, &[("User-Agent", UA)]).await {
        if let Some(date) = extract_date_from_json(&html) {
            return date;
        }
    }
    now()
}

fn extract_date_from_json(html: &str) -> Option<String> {
    // Look for "datePublished":"2026-08-01T14:00:00" in embedded JSON
    let mut start = 0;
    while let Some(idx) = html[start..].find("\"datePublished\":\"") {
        let pos = start + idx + "\"datePublished\":\"".len();
        if let Some(end) = html[pos..].find('"') {
            let raw = &html[pos..pos + end];
            if let Ok(naive) = NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S") {
                return Some(
                    DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
                        .format("%a, %d %b %Y %H:%M:%S %z")
                        .to_string(),
                );
            }
        }
        start = pos + 1;
    }
    None
}
