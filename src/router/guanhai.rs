use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let html = fetch_reqwest_get_with_headers(
        "https://www.guanhai.com.cn",
        &[(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )],
    ).await?;

    let links: Vec<String> = {
        let doc = Html::parse_document(&html);
        let mut links: Vec<String> = Vec::new();

        for a in doc.select(&Selector::parse(".img-box ul > a").unwrap()) {
            if let Some(href) = a.value().attr("href") {
                let title = a.value().attr("title").unwrap_or("");
                if !title.is_empty() && !links.contains(&href.to_string()) {
                    links.push(href.to_string());
                }
            }
        }

        for a in doc.select(&Selector::parse(".pic-summary .title a").unwrap()) {
            if let Some(href) = a.value().attr("href") {
                let title = a.text().collect::<String>().trim().to_string();
                if !title.is_empty()
                    && href.starts_with("http")
                    && !links.contains(&href.to_string())
                {
                    links.push(href.to_string());
                }
            }
        }
        links
    };

    if links.is_empty() {
        return Err(anyhow!("找不到文章链接"));
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
                    .select(&Selector::parse(".source").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
                let description = detail_doc
                    .select(&Selector::parse(".article-content").unwrap())
                    .next()
                    .map(|e| e.inner_html())
                    .or_else(|| {
                        detail_doc
                            .select(&Selector::parse(".video-content").unwrap())
                            .next()
                            .map(|e| e.inner_html())
                    })
                    .unwrap_or_default();

                if title.is_empty() {
                    continue;
                }

                let rss_item = ItemBuilder::default()
                    .title(Some(title))
                    .link(Some(link.clone()))
                    .author(Some(author))
                    .description(Some(description))
                    .build();
                item_vec.push(rss_item);
            }
            Err(_) => continue,
        }
    }

    let channel = ChannelBuilder::default()
        .title("观海新闻 - 首页")
        .link("https://www.guanhai.com.cn")
        .description("观海新闻首页推荐".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
