use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const MAINTAINER: &str = "AI转写 / huiinyg审核";

fn parse_list_date(datetime: &str) -> Option<String> {
    datetime_str_to_rss(&datetime[..19])
}

fn parse_detail(html: &str, link: &str) -> Result<(String, String, String)> {
    let doc = Html::parse_document(html);
    let sel_h1 = Selector::parse("article h1").map_err(|e| anyhow!("selector: {}", e))?;
    let sel_time = Selector::parse("article time").map_err(|e| anyhow!("selector: {}", e))?;
    let sel_content = Selector::parse("article div#content").map_err(|e| anyhow!("selector: {}", e))?;

    let title = doc
        .select(&sel_h1)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let pub_date = doc
        .select(&sel_time)
        .next()
        .and_then(|e| e.value().attr("datetime"))
        .and_then(parse_list_date)
        .unwrap_or_else(now);

    let content = doc
        .select(&sel_content)
        .next()
        .map(|e| {
            let mut inner = e.inner_html();
            inner = inner
                .replace("href=#", &format!("href={}#", link))
                .replace("href=/", "href=https://www.kali.org/");
            inner
        })
        .unwrap_or_default();

    Ok((title, pub_date, content))
}

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(50);

    let url = "https://www.kali.org/blog/";
    let html = fetch_reqwest_get_with_headers(
        url,
        &[
            ("User-Agent", UA),
            ("Accept", "text/html,application/xhtml+xml"),
        ],
    )
    .await?;

    let links: Vec<(String, String, String)> = {
        let doc = Html::parse_document(&html);
        let sel_article = Selector::parse("article").map_err(|e| anyhow!("selector: {}", e))?;

        let mut links: Vec<(String, String, String)> = Vec::new();
        for art in doc.select(&sel_article).take(limit) {
            let link = art
                .select(&Selector::parse("a").unwrap())
                .find_map(|a| a.value().attr("href"))
                .map(|s| s.to_string())
                .unwrap_or_default();
            let title = art
                .select(&Selector::parse("h1").unwrap())
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
            let pub_date = art
                .select(&Selector::parse("time").unwrap())
                .next()
                .and_then(|e| e.value().attr("datetime"))
                .and_then(parse_list_date)
                .unwrap_or_else(now);
            if link.is_empty() {
                continue;
            }
            links.push((link, title, pub_date));
        }
        links
    };

    if links.is_empty() {
        return Err(anyhow!("在 Kali blog 中找不到文章链接"));
    }

    let mut item_vec = Vec::new();
    for (link, list_title, list_date) in links {
        match fetch_reqwest_get_with_headers(
            &link,
            &[
                ("User-Agent", UA),
                ("Accept", "text/html,application/xhtml+xml"),
            ],
        )
        .await
        {
            Ok(detail_html) => {
                if let Ok((title, pub_date, content)) = parse_detail(&detail_html, &link) {
                    let final_title = if title.is_empty() { list_title } else { title };
                    if final_title.is_empty() {
                        continue;
                    }
                    let detail_doc = Html::parse_document(&detail_html);
                    let mut desc = String::new();
                    if let Some(img) = detail_doc
                        .select(&Selector::parse("article img").unwrap())
                        .next()
                        .and_then(|e| e.value().attr("src"))
                    {
                        desc.push_str(&format!("<figure><img src=\"{}\"></figure>", img));
                    }
                    desc.push_str(&content);
                    item_vec.push(
                        ItemBuilder::default()
                            .title(Some(final_title))
                            .link(link.clone())
                            .guid(Some(rss::Guid {
                                value: link.clone(),
                                permalink: true,
                            }))
                            .description(Some(desc))
                            .pub_date(if pub_date.is_empty() {
                                list_date
                            } else {
                                pub_date
                            })
                            .build(),
                    );
                }
            }
            Err(_) => continue,
        }
    }

    if item_vec.is_empty() {
        return Err(anyhow!("没有抓取到任何 Kali 博客文章"));
    }

    let channel = ChannelBuilder::default()
        .title("Kali Linux Blog")
        .link("https://www.kali.org/blog/".to_string())
        .description(format!(
            "Kali Linux 官方博客更新 | {} (遵守 robots.txt)",
            MAINTAINER
        ))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}