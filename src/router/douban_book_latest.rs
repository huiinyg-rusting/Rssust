use crate::easyuser::*;
use anyhow::{Error, Result};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const SUBCATS: &[(&str, &str)] = &[
    ("all", "全部"),
    ("prose_poetry", "文学"),
    ("fiction", "小说"),
    ("history", "历史文化"),
    ("biography", "社会纪实"),
    ("science", "科学新知"),
    ("art", "艺术设计"),
    ("business", "商业经管"),
    ("comics", "绘本漫画"),
];

fn subcat_name(t: &str) -> &str {
    for &(k, v) in SUBCATS {
        if k == t {
            return v;
        }
    }
    "全部"
}

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let book_type = para.get("type").map(|s| s.as_str()).unwrap_or("all");

    let url = format!(
        "https://m.douban.com/rexxar/api/v2/subject_collection/new_book_{}/items?start=0&count=10&mode=collection&for_mobile=1",
        book_type
    );

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &[("Referer", "https://book.douban.com/latest")])?.as_str(),
    )?;

    let items = json
        .pointer("/items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("找不到 items 字段或不是数组"))?;

    let name = subcat_name(book_type);
    let title_suffix = if book_type == "all" { String::new() } else { format!("-{}", name) };

    let mut item_vec = Vec::new();
    for item in items {
        let title = item["title"].as_str().unwrap_or("");
        let url = item["url"].as_str().unwrap_or("");
        let card_subtitle = item["card_subtitle"].as_str().unwrap_or("");
        let cards_content = item["cards"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|c| c["content"].as_str())
            .unwrap_or("");
        let pic = item["pic"]["normal"].as_str().unwrap_or("");
        let rating = item["rating"]["value"].as_f64();
        let null_reason = item["null_rating_reason"].as_str().unwrap_or("");

        let rate = match rating {
            Some(v) => format!("{}分", v),
            None => null_reason.to_string(),
        };

        let description = format!(
            "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>{}<br><br>{}<br><br>{}<br><br>{}",
            pic, title, card_subtitle, cards_content, rate
        );

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(url.to_string())
            .description(description)
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("豆瓣新书速递{}", title_suffix))
        .link(format!("https://book.douban.com/latest{}", if book_type == "all" { String::new() } else { format!("?subcat={}", name) }))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
