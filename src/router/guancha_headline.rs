use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let root_url = "https://www.guancha.cn";
    let list_url = format!("{}/GuanChaZheTouTiao/list_1.shtml", root_url);

    let html = fetch_reqwest_get(&list_url)?;
    let doc = Html::parse_document(&html);

    let sel = Selector::parse(".headline-list li .content-headline h3 a")
        .map_err(|_| anyhow!("选择器无效"))?;

    let mut entries = Vec::new();
    for elem in doc.select(&sel) {
        let title = elem.text().collect::<String>().trim().to_string();
        let href = elem.value().attr("href").unwrap_or("");
        let link = if href.starts_with("http") {
            href.to_string()
        } else {
            format!("{}{}", root_url, href)
        };
        entries.push((title, link));
    }

    let mut item_vec = Vec::new();
    for (title, link) in &entries {
        if title.is_empty() || link.is_empty() {
            continue;
        }
        let detail_url = link.replace(".shtml", "_s.shtml");

        if let Ok(detail_html) = fetch_reqwest_get(&detail_url) {
            let detail_doc = Html::parse_document(&detail_html);

            let all_txt_sel = Selector::parse(".all-txt")
                .map_err(|_| anyhow!("选择器无效"))?;
            let description = detail_doc
                .select(&all_txt_sel)
                .next()
                .map(|e| e.inner_html())
                .unwrap_or_default();

            let pub_date = extract_guancha_date(&detail_html);

            let item = ItemBuilder::default()
                .title(Some(title.clone()))
                .link(Some(link.clone()))
                .pub_date(pub_date)
                .description(Some(description))
                .build();
            item_vec.push(item);
        }
    }

    let channel = ChannelBuilder::default()
        .title("观察者网 - 头条")
        .link(list_url)
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

fn extract_guancha_date(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("div.time").ok()?;
    let text = doc
        .select(&sel)
        .next()?
        .text()
        .collect::<String>();
    let re = Regex::new(r"(\d{4}-\d{2}-\d{2}\s*\d{2}:\d{2})").ok()?;
    let caps = re.captures(&text)?;
    datetime_str_to_rss(&format!("{}:00", &caps[1]))
}
