use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const BASE: &str = "https://www.cenc.ac.cn/";

///中国地震台网中心 最新地震：抓取官网首页"最新地震"区块（服务端渲染）。
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(50);

    let html = fetch_reqwest_get_with_headers(BASE, &[("User-Agent", UA)]).await?;

    let items: Vec<(String, String, String)> = {
        let doc = Html::parse_document(&html);
        // 定位"最新地震"区块：标题 span 后面的 ul
        let title_sel = Selector::parse("span.tw_mk_title").map_err(|e| anyhow!("{}", e))?;
        let mut block: Option<scraper::element_ref::ElementRef> = None;
        for node in doc.select(&title_sel) {
            if node.text().collect::<String>().contains("最新地震") {
                if let Some(ul) = node
                    .parent()
                    .and_then(|p| p.next_sibling())
                    .and_then(|s| s.next_sibling())
                    .and_then(|d| scraper::element_ref::ElementRef::wrap(d))
                    .and_then(|d| {
                        d.select(&Selector::parse("ul").ok()?).next()
                    })
                {
                    block = Some(ul);
                }
                break;
            }
        }
        let ul = block.ok_or_else(|| anyhow!("找不到最新地震区块"))?;
        let li_sel = Selector::parse("li").map_err(|e| anyhow!("{}", e))?;
        let date_sel = Selector::parse("span.index_art_date").map_err(|e| anyhow!("{}", e))?;
        let a_sel = Selector::parse("a").map_err(|e| anyhow!("{}", e))?;

        ul.select(&li_sel)
            .filter_map(|li| {
                let title = li
                    .select(&a_sel)
                    .next()
                    .and_then(|a| a.value().attr("title"))
                    .map(|t| t.trim().to_string())
                    .or_else(|| {
                        li.select(&a_sel)
                            .next()
                            .map(|a| a.text().collect::<String>().trim().to_string())
                    })?;
                if title.is_empty() {
                    return None;
                }
                let href = li
                    .select(&a_sel)
                    .next()
                    .and_then(|a| a.value().attr("href"))
                    .map(|h| h.trim().to_string())
                    .unwrap_or_default();
                let link = if href.starts_with("http") {
                    href
                } else {
                    format!("https://www.cenc.ac.cn{}", href)
                };
                let date = li
                    .select(&date_sel)
                    .next()
                    .map(|s| s.text().collect::<String>())
                    .unwrap_or_default()
                    .trim()
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string();
                Some((title, link, date))
            })
            .take(limit)
            .collect()
    };

    let mut item_vec = Vec::new();
    for (title, link, date) in &items {
        let pub_date = if date.is_empty() {
            now()
        } else {
            let (m, day) = if let Some((m, dd)) = date.split_once('-') {
                (m.to_string(), dd.to_string())
            } else {
                (date.clone(), date.clone())
            };
            datetime_str_to_rss(&format!("2026-{:0>2}-{:0>2} 00:00:00", m, day)).unwrap_or_else(now)
        };
        let rss_item = ItemBuilder::default()
            .title(Some(title.clone()))
            .link(link.clone())
            .description(Some(title.clone()))
            .pub_date(pub_date)
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("中国地震台网中心 最新地震")
        .link(BASE)
        .description("中国地震台网中心 最新地震速报")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
