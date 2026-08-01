use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let cat = para.get("cat").map(|s| s.as_str()).unwrap_or("depth");
    let url = format!("https://www.bjnews.com.cn/{}", cat);

    let html = fetch_reqwest_get_with_headers(
        &url,
        &[(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )],
    ).await?;

    let links: Vec<String> = {
        let doc = Html::parse_document(&html);
        let link_selector = Selector::parse("#waterfall-container .pin_demo > a").unwrap();

        let mut links: Vec<String> = Vec::new();
        for a in doc.select(&link_selector) {
            if let Some(href) = a.value().attr("href") {
                let full_url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("https://www.bjnews.com.cn{}", href)
                };
                if !links.contains(&full_url) {
                    links.push(full_url);
                }
            }
        }
        links
    };

    if links.is_empty() {
        return Err(anyhow!("在 {} 中找不到文章链接", url));
    }

    let mut item_vec = Vec::new();
    for link in &links {
        match fetch_reqwest_get_with_headers(
            link,
            &[(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )],
        )
        .await
        {
            Ok(detail_html) => {
                let detail_doc = Html::parse_document(&detail_html);

                let title = detail_doc
                    .select(&Selector::parse("title").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
                let author = detail_doc
                    .select(&Selector::parse(".left-info .reporter").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
                let description = detail_doc
                    .select(&Selector::parse("#contentStr").unwrap())
                    .next()
                    .map(|e| e.inner_html())
                    .unwrap_or_default();

                let mut pub_date = now();
                if let Some(date_el) = detail_doc
                    .select(&Selector::parse(".left-info .timer").unwrap())
                    .next()
                {
                    let date_str = date_el.text().collect::<String>().trim().to_string();
                    if !date_str.is_empty() {
                        if let Some(d) = datetime_str_to_rss(&date_str) {
                            pub_date = d;
                        }
                    }
                }

                if title.is_empty() {
                    continue;
                }

                let rss_item = ItemBuilder::default()
                    .title(Some(title))
                    .link(Some(link.clone()))
                    .author(Some(author))
                    .pub_date(pub_date)
                    .description(Some(description))
                    .build();
                item_vec.push(rss_item);
            }
            Err(_) => continue,
        }
    }

    let channel = ChannelBuilder::default()
        .title(format!("新京报 - {}", cat))
        .link(url)
        .description("新京报分类文章".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
