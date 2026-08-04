use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const BASE: &str = "https://www.12306.cn/mormhweb/zxdt/index_zxdt.html";

///12306 最新动态：抓取 mormhweb/zxdt 公告列表。
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(50);

    let html = fetch_reqwest_get_with_headers(BASE, &[("User-Agent", UA)]).await?;

    let items: Vec<(String, String, String)> = {
        let doc = Html::parse_document(&html);
        let list_sel = Selector::parse("#newList ul li").map_err(|e| anyhow!("{}", e))?;
        let a_sel = Selector::parse("a").map_err(|e| anyhow!("{}", e))?;
        let time_sel = Selector::parse("span.zxdt_time_in").map_err(|e| anyhow!("{}", e))?;
        doc.select(&list_sel)
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
                    format!("https://www.12306.cn/mormhweb/zxdt/{}", href.trim_start_matches("./"))
                };
                let date = li
                    .select(&time_sel)
                    .next()
                    .map(|s| s.text().collect::<String>())
                    .unwrap_or_default();
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
            let d = date
                .trim_start_matches('(')
                .trim_end_matches(')')
                .trim()
                .to_string();
            datetime_str_to_rss(&format!("{} 00:00:00", d)).unwrap_or_else(now)
        };
        let rss_item = ItemBuilder::default()
            .title(Some(title.clone()))
            .link(link.clone())
            .description(Some("12306 最新动态".to_string()))
            .pub_date(pub_date)
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("12306 最新动态")
        .link(BASE)
        .description("中国铁路客户服务中心最新动态")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
