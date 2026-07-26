use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

const CATEGORIES: &[(&str, &str)] = &[
    ("社会", "shehui"),
    ("娱乐", "yule"),
    ("国际", "guoji"),
    ("军事", "junshi"),
    ("养生", "yangsheng"),
    ("汽车", "qiche"),
    ("体育", "tiyu"),
    ("财经", "caijing"),
    ("游戏", "youxi"),
    ("科技", "keji"),
    ("国内", "guonei"),
    ("宠物", "chongwu"),
    ("情感", "qinggan"),
    ("人文", "renwen"),
    ("教育", "jiaoyu"),
];

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let cat_name = para.get("category").map(|s| s.as_str()).unwrap_or("社会");
    let cat_slug = CATEGORIES
        .iter()
        .find(|(n, _)| *n == cat_name)
        .map(|(_, s)| *s)
        .unwrap_or("shehui");

    let api_url = format!(
        "https://mini.eastday.com/ns/api/detail/trust/trust-news-{}.json",
        cat_slug
    );

    let resp = fetch_reqwest_get(&api_url)?;

    let json_str = resp
        .strip_prefix("trustNews(")
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| anyhow!("JSONP 格式错误"))?;

    let json: Value = serde_json::from_str(json_str)?;
    let list = json["data"]["trust"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 data.trust"))?;

    let root_url = "https://mini.eastday.com";

    let mut item_vec = Vec::new();
    for entry in list {
        let title = entry["topic"].as_str().unwrap_or("");
        let url_path = entry["url"].as_str().unwrap_or("");

        if title.is_empty() || url_path.is_empty() {
            continue;
        }

        let link = format!("{}{}", root_url, url_path);

        let mut pub_date = now();
        let mut description = String::new();

        if let Ok(detail_html) = fetch_reqwest_get_with_headers(
            &link,
            &[("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")],
        ) {
            let detail_doc = Html::parse_document(&detail_html);

            if let Some(content) = detail_doc
                .select(&Selector::parse("#J-contain_detail_cnt").unwrap())
                .next()
            {
                description = content.inner_html();
            }

            if let Some(meta) = detail_doc
                .select(&Selector::parse("meta[property='og:release_date']").unwrap())
                .next()
            {
                if let Some(date_str) = meta.value().attr("content") {
                    let dt = date_str.replace('T', " ").replace('Z', "");
                    if let Some(d) = datetime_str_to_rss(&dt) {
                        pub_date = d;
                    }
                }
            }

            // handle pagination
            let detail_text = fetch_reqwest_get_with_headers(
                &link,
                &[("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")],
            )
            .unwrap_or_default();

            if let Some(caps) = regex::Regex::new(r"var page_num = '(\d+)'")
                .unwrap()
                .captures(&detail_text)
            {
                if let Ok(page_num) = caps[1].parse::<i32>() {
                    if page_num > 1 {
                        for i in 2..=page_num {
                            let page_link = if link.ends_with(".html") {
                                format!("{}-{}.html", &link[..link.len() - 5], i)
                            } else {
                                link.clone()
                            };
                            if let Ok(page_html) = fetch_reqwest_get_with_headers(
                                &page_link,
                                &[("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")],
                            ) {
                                let page_doc = Html::parse_document(&page_html);
                                if let Some(page_content) = page_doc
                                    .select(&Selector::parse("#J-contain_detail_cnt").unwrap())
                                    .next()
                                {
                                    description.push_str(&page_content.inner_html());
                                }
                            }
                        }
                    }
                }
            }
        }

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(Some(link))
            .pub_date(pub_date)
            .description(Some(description))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("24小时{}热闻 - 东方资讯", cat_name))
        .link(format!("{}/#{}", root_url, cat_slug))
        .description("东方资讯24小时热闻".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
