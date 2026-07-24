use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let html = fetch_reqwest_get_with_headers(
        "https://www.jianshu.com",
        &[("Referer", "https://www.jianshu.com")],
    )?;
    let doc = Html::parse_document(&html);

    let sel = Selector::parse(".note-list li")
        .map_err(|_| anyhow!("选择器无效"))?;

    let mut entries = Vec::new();
    for elem in doc.select(&sel) {
        let title_sel = Selector::parse(".title")
            .map_err(|_| anyhow!("选择器无效"))?;
        let name_sel = Selector::parse(".nickname")
            .map_err(|_| anyhow!("选择器无效"))?;

        let title = elem
            .select(&title_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let href = elem
            .select(&title_sel)
            .next()
            .and_then(|e| e.value().attr("href"))
            .unwrap_or("");
        let link = if href.starts_with("http") {
            href.to_string()
        } else {
            format!("https://www.jianshu.com{}", href)
        };
        let author = elem
            .select(&name_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if !title.is_empty() {
            entries.push((title, link, author));
        }
    }

    let mut item_vec = Vec::new();
    for (title, link, author) in entries {
        if let Ok(detail_html) = fetch_reqwest_get(&link) {
            let detail_doc = Html::parse_document(&detail_html);

            let pub_date = extract_jianshu_date(&detail_html);

            let description = detail_doc
                .select(&Selector::parse("article").unwrap())
                .next()
                .map(|e| e.inner_html())
                .unwrap_or_default();

            let item = ItemBuilder::default()
                .title(Some(title))
                .link(Some(link))
                .pub_date(pub_date)
                .description(Some(description))
                .author(Some(author))
                .build();
            item_vec.push(item);
        }
    }

    let channel = ChannelBuilder::default()
        .title("简书首页")
        .link("https://www.jianshu.com")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

fn extract_jianshu_date(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("time").ok()?;
    let datetime = doc
        .select(&sel)
        .next()?
        .value()
        .attr("datetime")?;
    Some(
        chrono::DateTime::parse_from_rfc3339(datetime)
            .ok()
            .map(|dt| dt.to_rfc2822())
            .unwrap_or_default(),
    )
}
