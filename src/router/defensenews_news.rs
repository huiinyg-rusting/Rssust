use crate::easyuser::*;
use anyhow::{Error, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;
use tokio::sync::Semaphore;
use std::sync::Arc;

const MAINTAINER: &str = "huinyg / Defense News (defense & military news, no official RSS)";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let html = fetch_reqwest_get_with_headers("https://www.defensenews.com", &[("User-Agent", UA)])
        .await?;

    let cards = extract_cards(&html);
    tracing::info!("defensenews: extracted {} cards", cards.len());

    // Parallelize article date fetching with concurrency limit
    let semaphore = Arc::new(Semaphore::new(10));
    let mut handles = Vec::new();

    for (title, link, author) in cards {
        let semaphore = semaphore.clone();
        let ua = UA.to_string();
        let link_clone = link.clone();
        let title_clone = title.clone();
        let author_clone = author.clone();

        handles.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let pub_date = fetch_article_date(&link_clone, &ua).await;
            (title_clone, link_clone, author_clone, pub_date)
        }));
    }

    let mut items = Vec::new();
    for handle in handles {
        match handle.await {
            Ok((title, link, author, pub_date)) => {
                items.push(
                    ItemBuilder::default()
                        .title(Some(title))
                        .link(link)
                        .pub_date(pub_date)
                        .description(if author.is_empty() {
                            None
                        } else {
                            Some(format!("By {}", author))
                        })
                        .build(),
                );
            }
            Err(e) => {
                tracing::warn!("defensenews: task join error: {}", e);
            }
        }
    }
    tracing::info!("defensenews: built {} items", items.len());

    let channel = ChannelBuilder::default()
        .title("Defense News - Global Defense & Military News")
        .link("https://www.defensenews.com")
        .description(format!(
            "Defense News latest articles - global defense, military & aerospace (no official RSS) | {}",
            MAINTAINER
        ))
        .items(items)
        .build();

    Ok(channel.to_string())
}

fn extract_cards(html: &str) -> Vec<(String, String, String)> {
    let doc = Html::parse_document(html);
    // Article cards have data-story-url and itemType="http://schema.org/Article"
    let card_sel = Selector::parse(r#"article[data-story-url]"#).unwrap();
    // Headline is in h3/h4/h5 with itemProp="headline" or class o-storyCard__headline
    let title_sel = Selector::parse(r#"[itemProp="headline"], .o-storyCard__headline"#).unwrap();
    // Author byline
    let author_sel = Selector::parse(r#"span[class*="Byline__Author"]"#).unwrap();

    let mut cards = Vec::new();
    for card in doc.select(&card_sel) {
        // Try multiple selectors for title
        let title = card
            .select(&title_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let link = card
            .value()
            .attr("data-story-url")
            .map(|h| {
                if h.starts_with("http") {
                    h.to_string()
                } else {
                    format!("https://www.defensenews.com{}", h)
                }
            })
            .unwrap_or_default();

        if title.is_empty() || link.is_empty() {
            continue;
        }

        let author = card
            .select(&author_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        cards.push((title, link, author));

        if cards.len() >= 15 {
            break;
        }
    }

    cards
}

async fn fetch_article_date(link: &str, ua: &str) -> String {
    if let Ok(html) = fetch_reqwest_get_with_headers(link, &[("User-Agent", ua)]).await {
        if let Some(date) = extract_date_from_json(&html) {
            return date;
        }
    }
    now()
}

fn extract_date_from_json(html: &str) -> Option<String> {
    // Look for "datePublished":"2026-07-21T16:07:00.832Z" in embedded JSON
    let mut start = 0;
    while let Some(idx) = html[start..].find("\"datePublished\":\"") {
        let pos = start + idx + "\"datePublished\":\"".len();
        if let Some(end) = html[pos..].find('"') {
            let raw = &html[pos..pos + end];
            if let Ok(parsed) = DateTime::parse_from_rfc3339(raw) {
                return Some(
                    parsed
                        .with_timezone(&Utc)
                        .format("%a, %d %b %Y %H:%M:%S %z")
                        .to_string(),
                );
            }
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