use crate::easyuser::*;
use anyhow::{Error, Result};
use regex::Regex;
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let root_url = "https://www.chinanews.com.cn";
    let list_url = format!("{}/scroll-news/news1.html", root_url);
    let html = fetch_reqwest_get(&list_url)?;
    let doc = Html::parse_document(&html);

    let sel_item = Selector::parse("div.dd_bt a").unwrap();

    let mut item_vec = Vec::new();
    for a_el in doc.select(&sel_item) {
        let href = a_el.value().attr("href").unwrap_or("");
        if href.is_empty() || !href.ends_with(".shtml") {
            continue;
        }
        let title = a_el.text().collect::<String>().trim().to_string();
        if title.is_empty() {
            continue;
        }
        let link = if href.starts_with("http") {
            href.to_string()
        } else {
            format!("{}{}", root_url, href)
        };

        let mut description = String::new();
        let mut pub_date = now();
        let mut author = String::new();

        if let Ok(detail_html) = fetch_reqwest_get(&link) {
            let detail_doc = Html::parse_document(&detail_html);

            let sel_content_desc = Selector::parse("div.content_desc").unwrap();
            let sel_left_zw = Selector::parse("div.left_zw").unwrap();
            let sel_t3 = Selector::parse("div.t3").unwrap();

            if let Some(div) = detail_doc.select(&sel_content_desc).next() {
                description = div.inner_html();
            } else if let Some(div) = detail_doc.select(&sel_left_zw).next() {
                description = div.inner_html();
            } else if let Some(div) = detail_doc.select(&sel_t3).next() {
                description = div.inner_html();
            }

            let re_date = Regex::new(r"(\d{4}年\d{2}月\d{2}日\s*\d{2}:\d{2})").unwrap();
            if let Some(cap) = re_date.captures(&detail_html) {
                let date_str = cap.get(1).unwrap().as_str();
                pub_date = chinese_date_to_parse(date_str).unwrap_or_else(now);
            }

            let re_source = Regex::new(r"来源[：:]([^<]{2,20})").unwrap();
            if let Some(cap) = re_source.captures(&detail_html) {
                author = cap.get(1).unwrap().as_str().trim().to_string();
            }
        }

        if description.is_empty() {
            continue;
        }

        let rss_item = ItemBuilder::default()
            .title(Some(title))
            .link(Some(link))
            .pub_date(pub_date)
            .description(Some(description))
            .author(if author.is_empty() { None } else { Some(author) })
            .build();
        item_vec.push(rss_item);

        if item_vec.len() >= 30 {
            break;
        }
    }

    let channel = ChannelBuilder::default()
        .title("中国新闻网 - 滚动新闻")
        .link(list_url)
        .description("中国新闻网滚动新闻".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
