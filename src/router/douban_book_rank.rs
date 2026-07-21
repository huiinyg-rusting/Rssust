use crate::easyuser::*;
use anyhow::{Error, Result};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

fn fetch_items(book_type: &str, referer: &str) -> Result<Vec<Value>, Error> {
    let url = format!(
        "https://m.douban.com/rexxar/api/v2/subject_collection/book_{}/items?start=0&count=10",
        book_type
    );
    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &[("Referer", referer)])?.as_str(),
    )?;
    Ok(json
        .pointer("/subject_collection_items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default())
}

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let book_type = para.get("type").map(|s| s.as_str()).unwrap_or("");

    let referer = format!("https://m.douban.com/book/{}", book_type);

    let items: Vec<Value> = if !book_type.is_empty() {
        fetch_items(book_type, &referer)?
    } else {
        let mut all = fetch_items("fiction", &referer)?;
        all.extend(fetch_items("nonfiction", &referer)?);
        all
    };

    let type_name = match book_type {
        "fiction" => "虚构类",
        "nonfiction" => "非虚构类",
        _ => "全部",
    };

    let mut item_vec = Vec::new();
    for item in items {
        let title = item["title"].as_str().unwrap_or("");
        let url = item["url"].as_str().unwrap_or("");
        let cover = item["cover"]["url"].as_str().unwrap_or("");
        let info = item["info"].as_str().unwrap_or("");
        let rating = item["rating"]["value"].as_f64();
        let null_reason = item["null_rating_reason"].as_str().unwrap_or("");

        let rate = match rating {
            Some(v) => format!("{:.1}分", v),
            None => null_reason.to_string(),
        };

        let description = format!(
            "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>{} / {} / {}",
            cover, title, info, rate
        );

        let rss_item = ItemBuilder::default()
            .title(Some(format!("{}-{}", title, info)))
            .link(url.to_string())
            .description(description)
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("豆瓣热门图书-{}", type_name))
        .link(referer)
        .description("每周一更新")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
