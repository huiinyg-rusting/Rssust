use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const TYPES: &[(&str, &str, &str)] = &[
    ("24h", "d-1", "24 小时最热"),
    ("7days", "d-2", "7 天最热"),
    ("monthly", "d-3", "月榜"),
];

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let type_ = para.get("type").map(|s| s.as_str()).unwrap_or("24h");
    let (id, title) = TYPES
        .iter()
        .find(|(k, _, _)| *k == type_)
        .map(|(_, id, t)| (*id, *t))
        .ok_or_else(|| anyhow!("未知类型：{}", type_))?;

    let html = fetch_reqwest_get("https://www.ithome.com/block/rank.html")?;
    let doc = Html::parse_document(&html);

    let sel = Selector::parse(&format!("#{} > li a", id))
        .map_err(|_| anyhow!("选择器无效"))?;

    let mut entries = Vec::new();
    for elem in doc.select(&sel) {
        let title = elem.text().collect::<String>().trim().to_string();
        let href = elem.value().attr("href").unwrap_or("");
        if !title.is_empty() && !href.is_empty() {
            entries.push((title, href.to_string()));
        }
    }

    let mut item_vec = Vec::new();
    for (title, link) in entries {
        if let Ok(detail_html) = fetch_reqwest_get(&link) {
            let detail_doc = Html::parse_document(&detail_html);

            let content_sel = Selector::parse("#paragraph")
                .map_err(|_| anyhow!("选择器无效"))?;
            let description = detail_doc
                .select(&content_sel)
                .next()
                .map(|e| e.inner_html())
                .unwrap_or_default();

            let pub_date = extract_ithome_date(&detail_html);

            let item = ItemBuilder::default()
                .title(Some(title))
                .link(Some(link))
                .pub_date(pub_date)
                .description(Some(description))
                .build();
            item_vec.push(item);
        }
    }

    let channel = ChannelBuilder::default()
        .title(format!("IT之家-{}", title))
        .link("https://www.ithome.com")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

fn extract_ithome_date(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("#pubtime_baidu").ok()?;
    let text = doc
        .select(&sel)
        .next()?
        .text()
        .collect::<String>();
    let re = Regex::new(r"(\d{4}-\d{2}-\d{2}\s*\d{2}:\d{2})").ok()?;
    let caps = re.captures(&text)?;
    datetime_str_to_rss(&format!("{}:00", &caps[1]))
}
