use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/* 简书首页服务端渲染已大幅缩水：主列表只出 1 篇，另有「今日热点」区块若干标题链接。
文章正文与发布时间改为前端调接口获取，这里直接请求同一接口：
https://www.jianshu.com/asimov/p/{slug} */
pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let html = fetch_reqwest_get_with_headers(
        "https://www.jianshu.com",
        &[
            ("User-Agent", UA_CHROME),
            ("Referer", "https://www.jianshu.com"),
        ],
    )
    .await?;

    let entries: Vec<(String, String)> = {
        let doc = Html::parse_document(&html);

        let note_sel = Selector::parse(".note-list .title").map_err(|_| anyhow!("选择器无效"))?;
        let hot_sel =
            Selector::parse(".today-selection-list a").map_err(|_| anyhow!("选择器无效"))?;

        let mut entries = Vec::new();
        let mut seen = HashSet::new();
        for elem in doc.select(&note_sel).chain(doc.select(&hot_sel)) {
            let Some(slug) = elem
                .value()
                .attr("href")
                .and_then(|h| h.strip_prefix("/p/"))
            else {
                continue;
            };
            let title = elem.text().collect::<String>().trim().to_string();
            if !title.is_empty() && seen.insert(slug.to_string()) {
                entries.push((slug.to_string(), title));
            }
        }
        entries.truncate(15);
        entries
    };

    let mut item_vec = Vec::new();
    for (slug, page_title) in entries {
        let link = format!("https://www.jianshu.com/p/{}", slug);

        /* 接口失败时退化为仅标题条目，避免整条路由空转 */
        let detail = fetch_reqwest_get_with_headers(
            &format!("https://www.jianshu.com/asimov/p/{}", slug),
            &[
                ("User-Agent", UA_CHROME),
                ("Referer", link.as_str()),
                ("Accept", "application/json"),
            ],
        )
        .await
        .ok()
        .and_then(|body| serde_json::from_str::<Value>(&body).ok());

        let (title, description, pub_date, author) = match detail {
            Some(json) => {
                let title = json["public_title"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .unwrap_or(&page_title)
                    .to_string();
                let description = json["free_content"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                let pub_date = json["first_shared_at"].as_str().map(rfc3339_to_rss);
                let author = json
                    .pointer("/user/nickname")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                (title, description, pub_date, author)
            }
            None => (page_title, String::new(), None, String::new()),
        };

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(Some(link))
            .pub_date(pub_date)
            .description(Some(description))
            .author(Some(author))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title("简书首页")
        .link("https://www.jianshu.com")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
