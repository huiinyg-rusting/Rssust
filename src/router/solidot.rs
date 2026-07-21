use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const MAINTAINER: &str = "AI转写 / huinyg审核 - 来源于RSShub@sgqy, hang333, TonyRL";

fn parse_article_date(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("div.talk_time").ok()?;
    let el = doc.select(&sel).next()?;

    let _date_raw: String = el.text().collect();
    let base = el.html().to_string();

    let re = Regex::new(r"(\d{4})年(\d{1,2})月(\d{1,2})日(\d{1,2})时(\d{1,2})分").ok()?;
    let caps = re.captures(&base)?;

    let formatted = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:00",
        caps[1].parse::<i32>().ok()?,
        caps[2].parse::<u32>().ok()?,
        caps[3].parse::<u32>().ok()?,
        caps[4].parse::<u32>().ok()?,
        caps[5].parse::<u32>().ok()?,
    );
    datetime_str_to_rss(&formatted)
}

fn parse_article(html: &str, url: &str) -> Result<rss::Item> {
    let doc = Html::parse_document(html);
    let sel_h2 = Selector::parse("div.ct_tittle div.bg_htit h2").map_err(|_| anyhow!("selector"))?;
    let sel_block = Selector::parse("div.block_m").map_err(|_| anyhow!("selector"))?;
    let sel_author = Selector::parse("div.talk_time b").map_err(|_| anyhow!("selector"))?;
    let sel_cat = Selector::parse("div.icon_float a").map_err(|_| anyhow!("selector"))?;

    let title = doc
        .select(&sel_h2)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let author = doc
        .select(&sel_author)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let _category = doc
        .select(&sel_cat)
        .next()
        .and_then(|e| e.value().attr("title"))
        .unwrap_or("")
        .to_string();

    let pub_date = parse_article_date(html).unwrap_or_else(now);

    let mut description = String::new();
    if let Some(block) = doc.select(&sel_block).next() {
        let inner = block.inner_html();
        let inner = inner
            .replace("href=\"/", &format!("href=\"https://www.solidot.org/"))
            .replace("<u>", "")
            .replace("</u>", "");
        description = inner;
    }

    let item = ItemBuilder::default()
        .title(Some(title))
        .link(url.to_string())
        .pub_date(pub_date)
        .description(Some(description))
        .author(Some(author))
        .build();
    Ok(item)
}

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let type_ = para.get("type").map(|s| s.as_str()).unwrap_or("www");
    let base_url = format!("https://{}.solidot.org", type_);

    let html = fetch_reqwest_get_with_headers(
        &base_url,
        &[(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )],
    )?;
    let doc = Html::parse_document(&html);

    let sel = Selector::parse("div.block_m div.bg_htit h2 a").map_err(|_| anyhow!("selector"))?;
    let urls: Vec<String> = doc
        .select(&sel)
        .filter_map(|e| e.value().attr("href"))
        .map(|href| {
            if href.starts_with("http") {
                href.to_string()
            } else {
                format!("https://www.solidot.org{}", href)
            }
        })
        .collect();

    let mut item_vec = Vec::new();
    for url in urls {
        match fetch_reqwest_get_with_headers(
            &url,
            &[(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            )],
        ) {
            Ok(article_html) => {
                if let Ok(item) = parse_article(&article_html, &url) {
                    item_vec.push(item);
                }
            }
            Err(_) => continue,
        }
    }

    let channel = ChannelBuilder::default()
        .title("奇客的资讯，重要的东西")
        .link(base_url)
        .description(format!("Solidot {} 最新消息 | {}", type_, MAINTAINER))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
