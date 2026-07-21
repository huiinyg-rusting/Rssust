use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let html = fetch_reqwest_get_with_headers(
        "https://m.thepaper.cn",
        &[("Cookie", "blackAndWhiteMode=0; redTops=0")],
    )?;

    let doc = Html::parse_document(&html);
    let sel = Selector::parse(r#"script[id="__NEXT_DATA__"]"#)
        .map_err(|_| anyhow!("选择器无效"))?;
    let json_str = doc
        .select(&sel)
        .next()
        .map(|el| el.text().collect::<String>())
        .ok_or_else(|| anyhow!("找不到 __NEXT_DATA__ script 标签"))?;

    let next_data: Value = serde_json::from_str(&json_str)?;

    let list = next_data
        .pointer("/props/pageProps/data/list")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 list"))?;

    let top_list = next_data
        .pointer("/props/pageProps/topData/recommendImg")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 recommendImg"))?;

    let mut item_vec = Vec::new();
    for item in list.iter().chain(top_list.iter()) {
        let title = item["name"].as_str().unwrap_or("");
        let cont_id = item["contId"].as_str().unwrap_or("");
        let link = format!("https://m.thepaper.cn/detail/{}", cont_id);
        let pub_ts = item["pubTimeLong"].as_i64().unwrap_or(0) / 1000;
        let pub_date = timestamp_to_rss(pub_ts);
        let description = item["name"].as_str().unwrap_or("");

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(link)
            .pub_date(pub_date)
            .description(Some(description.to_string()))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("澎湃新闻 - 首页头条")
        .link("https://m.thepaper.cn")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
