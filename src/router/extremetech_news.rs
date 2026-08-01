use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::{DateTime, Utc};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const MAINTAINER: &str = "huinyg / ExtremeTech (no working official RSS)";

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let html = fetch_reqwest_get_with_headers(
        "https://www.extremetech.com",
        &[("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")],
    ).await?;

    let doc = Html::parse_document(&html);
    let item_sel = Selector::parse("div.item.mt-4.flex").map_err(|_| anyhow!("item selector"))?;
    let title_link_sel =
        Selector::parse("div.w-3\\/4 > a.block").map_err(|_| anyhow!("title link selector"))?;
    let desc_sel = Selector::parse("div.mt-2.hidden.w-full.text-gray-600.md\\:block")
        .map_err(|_| anyhow!("desc selector"))?;
    let time_sel = Selector::parse("time[datetime]").map_err(|_| anyhow!("time selector"))?;

    let mut item_vec = Vec::new();

    for item in doc.select(&item_sel).take(15) {
        let title_link = item.select(&title_link_sel).next();
        let title = title_link
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let link = title_link
            .and_then(|e| e.value().attr("href"))
            .map(|h| {
                if h.starts_with("http") {
                    h.to_string()
                } else {
                    format!("https://www.extremetech.com{}", h)
                }
            })
            .unwrap_or_default();

        let description = item
            .select(&desc_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let time_elem = item.select(&time_sel).next();
        let pub_date = time_elem
            .and_then(|e| e.value().attr("datetime"))
            .and_then(|dt| {
                DateTime::parse_from_rfc2822(dt)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            })
            .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
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
        .title("ExtremeTech - Latest Tech News")
        .link("https://www.extremetech.com")
        .description(format!(
            "ExtremeTech latest articles (official RSS returns 403) | {}",
            MAINTAINER
        ))
        .items(item_vec)
        .build();

    Ok(channel.to_string())
}
