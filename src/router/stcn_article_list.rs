use crate::easyuser::*;
use anyhow::{Error, Result};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let id = para.get("id").map(|s| s.as_str()).unwrap_or("yw");
    let base_url = "https://www.stcn.com";
    let list_url = format!("{}/article/list/{}.html", base_url, id);

    let html = fetch_reqwest_get(&list_url)?;
    let doc = Html::parse_document(&html);

    let selector = Selector::parse("ul.infinite-list li").unwrap();
    let link_selector = Selector::parse("div.tt a").unwrap();
    let text_selector = Selector::parse("div.text").unwrap();
    let info_selector = Selector::parse("div.info span").unwrap();
    let tags_selector = Selector::parse("div.tags a").unwrap();

    let mut item_vec = Vec::new();
    for li in doc.select(&selector) {
        let title = li
            .select(&link_selector)
            .next()
            .map(|a| a.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let href = li
            .select(&link_selector)
            .next()
            .and_then(|a| a.value().attr("href"))
            .unwrap_or("");

        if title.is_empty() || href.is_empty() {
            continue;
        }

        let link = if href.starts_with("http") {
            href.to_string()
        } else {
            format!("{}{}", base_url, href)
        };

        let summary = li
            .select(&text_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let info_spans: Vec<String> = li
            .select(&info_selector)
            .map(|s| s.text().collect::<String>().trim().to_string())
            .collect();

        let mut author = String::new();
        let mut pub_date = now();
        if info_spans.len() >= 2 {
            author = info_spans[1].clone();
        }
        if let Some(t) = info_spans.last() {
            let clean = t.trim();
            if clean.len() == 5 && clean.contains(':') {
                pub_date = format!("{} {}:00 +0800", today_date_str(), clean);
            } else {
                if let Some(d) = datetime_str_to_rss(clean) {
                    pub_date = d;
                }
            }
        }

        let categories: Vec<String> = li
            .select(&tags_selector)
            .map(|e| e.text().collect::<String>().trim().to_string())
            .collect();

        let mut description = summary.clone();
        if let Ok(detail_html) = fetch_reqwest_get(&link) {
            let detail_doc = Html::parse_document(&detail_html);
            if let Some(content) = detail_doc
                .select(&Selector::parse("div.detail-content").unwrap())
                .next()
            {
                let inner = content.inner_html();
                let clean = inner
                    .split('<')
                    .filter(|p| !p.starts_with("script"))
                    .collect::<Vec<_>>()
                    .join("<");
                if !clean.is_empty() {
                    description = clean;
                }
            }
        }

        let author_str = if author.is_empty() { "证券时报" } else { &author };

        let cats: Vec<Category> = categories
            .iter()
            .map(|c| CategoryBuilder::default().name(c.clone()).build())
            .collect();

        let rss_item = ItemBuilder::default()
            .title(Some(title))
            .link(Some(link))
            .pub_date(pub_date)
            .author(Some(author_str.to_string()))
            .description(Some(description))
            .categories(cats)
            .build();

        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("证券时报 - {}", category_name(id)))
        .link(list_url)
        .description("证券时报网新闻列表".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

fn category_name(id: &str) -> &str {
    match id {
        "yw" => "要闻",
        "gs" => "股市",
        "company" => "公司",
        "fund" => "基金",
        "finance" => "金融",
        "comment" => "评论",
        "cj" => "产经",
        "kcb" => "科创板",
        "xsb" => "新三板",
        "zk" => "ESG",
        "gd" => "滚动",
        _ => id,
    }
}

fn today_date_str() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}
