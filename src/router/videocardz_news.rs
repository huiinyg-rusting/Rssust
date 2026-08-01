use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::{Datelike, TimeZone};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const MAINTAINER: &str = "huinyg / VideoCardz.com";
const BASE_URL: &str = "https://videocardz.com";

fn parse_time(time1: &str, time2: &str) -> Option<String> {
    let time1 = time1.trim();
    let time2 = time2.trim();
    let now = chrono::Local::now();
    let year = now.year();
    let month = match time2.split_whitespace().next() {
        Some("Jan") => 1,
        Some("Feb") => 2,
        Some("Mar") => 3,
        Some("Apr") => 4,
        Some("May") => 5,
        Some("Jun") => 6,
        Some("Jul") => 7,
        Some("Aug") => 8,
        Some("Sep") => 9,
        Some("Oct") => 10,
        Some("Nov") => 11,
        Some("Dec") => 12,
        _ => return None,
    };
    let day = time2.split_whitespace().nth(1)?.parse::<u32>().ok()?;
    let (hour, min) = time1.split_once(':')?;
    let hour = hour.parse::<u32>().ok()?;
    let min = min.parse::<u32>().ok()?;
    let naive = chrono::NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, min, 0)?;
    let fixed = chrono::FixedOffset::east_opt(8 * 3600)?;
    let dt = fixed.from_local_datetime(&naive).single()?;
    Some(dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
}

fn parse_entry(entry_html: &str) -> Result<(String, String, String)> {
    let doc = Html::parse_fragment(entry_html);

    let title_sel =
        Selector::parse("div.techbuzz-entry-title").map_err(|_| anyhow!("title selector"))?;
    let title = doc
        .select(&title_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .ok_or_else(|| anyhow!("no title"))?;

    let time1_sel =
        Selector::parse("div.techbuzz-entry-time1").map_err(|_| anyhow!("time1 selector"))?;
    let time2_sel =
        Selector::parse("div.techbuzz-entry-time2").map_err(|_| anyhow!("time2 selector"))?;
    let time1 = doc
        .select(&time1_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();
    let time2 = doc
        .select(&time2_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let link_sel = Selector::parse("a.techbuzz-entry.techbuzz-entry-index")
        .map_err(|_| anyhow!("link selector"))?;
    let link = doc
        .select(&link_sel)
        .next()
        .and_then(|e| e.value().attr("href"))
        .map(|h| h.to_string())
        .ok_or_else(|| anyhow!("no link"))?;

    Ok((title, link, format!("{} {}", time1, time2)))
}

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    let html = fetch_reqwest_get_with_headers(BASE_URL, &[("User-Agent", ua)]).await?;

    let doc = Html::parse_document(&html);
    let entry_sel = Selector::parse("a.techbuzz-entry.techbuzz-entry-index")
        .map_err(|_| anyhow!("entry selector"))?;

    let mut item_vec = Vec::new();

    for entry in doc.select(&entry_sel) {
        let entry_html = entry.html();
        let (title, link, time_str) = match parse_entry(&entry_html) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let pub_date = parse_time(
            &time_str.split_whitespace().next().unwrap_or("00:00"),
            &time_str
                .split_whitespace()
                .skip(1)
                .collect::<Vec<_>>()
                .join(" "),
        )
        .unwrap_or_else(now);

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(link)
            .pub_date(pub_date)
            .build();

        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title("VideoCardz - Latest GPU & Hardware News")
        .link(BASE_URL)
        .description(format!("VideoCardz.com latest news | {}", MAINTAINER))
        .items(item_vec)
        .build();

    Ok(channel.to_string())
}
