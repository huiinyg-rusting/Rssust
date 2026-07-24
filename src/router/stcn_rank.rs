use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let id = para.get("id").map(|s| s.as_str()).unwrap_or("yw");
    let base_url = "https://www.stcn.com";

    let json_str = fetch_reqwest_get_with_headers(
        &format!(
            "https://www.stcn.com/article/category-news-rank.html?type={}",
            id
        ),
        &[("X-Requested-With", "XMLHttpRequest")],
    )?;

    let json: Value = serde_json::from_str(&json_str)?;
    let list = json["data"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 data 字段"))?;

    let mut item_vec = Vec::new();
    for article in list {
        let title = article["title"].as_str().unwrap_or("");
        let url_path = article["url"].as_str().unwrap_or("");

        if title.is_empty() || url_path.is_empty() {
            continue;
        }

        let link = format!("{}{}", base_url, url_path);

        let mut description = String::new();
        let mut pub_date = now();
        let mut author = String::new();
        let mut categories: Vec<String> = Vec::new();

        if let Ok(detail_html) = fetch_reqwest_get(&link) {
            let detail_doc = Html::parse_document(&detail_html);

            if let Some(dc) = detail_doc
                .select(&Selector::parse("div.detail-content").unwrap())
                .next()
            {
                let inner = dc.inner_html();
                let clean: String = inner
                    .split('<')
                    .filter(|p| !p.starts_with("script"))
                    .collect::<Vec<_>>()
                    .join("<");
                description = clean;
            }

            if let Some(info) = detail_doc
                .select(&Selector::parse("div.detail-info").unwrap())
                .next()
            {
                let spans: Vec<String> = info
                    .select(&Selector::parse("span").unwrap())
                    .map(|s| s.text().collect::<String>().trim().to_string())
                    .collect();
                if spans.len() >= 3 {
                    author = spans[1]
                        .strip_prefix("作者：")
                        .unwrap_or(&spans[1])
                        .to_string();
                    if let Some(d) = datetime_str_to_rss(&spans[2]) {
                        pub_date = d;
                    }
                }
            }

            if let Some(meta) = detail_doc
                .select(&Selector::parse("meta[name='keywords']").unwrap())
                .next()
            {
                if let Some(content) = meta.value().attr("content") {
                    categories = content.split(',').map(|s| s.trim().to_string()).collect();
                }
            }
        }

        let cats: Vec<Category> = categories
            .iter()
            .map(|c| CategoryBuilder::default().name(c.clone()).build())
            .collect();

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(Some(link))
            .pub_date(pub_date)
            .author(Some(author))
            .description(Some(description))
            .categories(cats)
            .build();

        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("证券时报 - {}热榜", category_name(id)))
        .link(format!("https://www.stcn.com/article/list/{}.html", id))
        .description("证券时报网热榜".to_string())
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
