use crate::easyuser::*;
use anyhow::{Error, Result};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let sort = para.get("sort").map(|s| s.as_str()).unwrap_or("U");
    let score = para
        .get("score")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let tags = para.get("tags").map(|s| s.as_str()).unwrap_or("");

    let url = format!(
        "https://movie.douban.com/j/new_search_subjects?sort={}&range=0,10&tags={}&start=0",
        sort, tags
    );

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &[("Referer", "https://movie.douban.com/tag/")])?.as_str(),
    )?;

    let movies = json
        .pointer("/data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("找不到 data 字段或不是数组"))?;

    let title = format!("豆瓣电影分类{}影视", if score > 0.0 { format!("超过 {} 分的", score) } else { String::new() });

    let mut item_vec = Vec::new();
    for item in movies {
        let rate_str = item["rate"].as_str().unwrap_or("");
        if !rate_str.is_empty() {
            if let Ok(score_val) = rate_str.parse::<f64>() {
                if score_val < score {
                    continue;
                }
            }
        }

        let title = item["title"].as_str().unwrap_or("");
        let rate = item["rate"].as_str().unwrap_or("");
        let directors = item["directors"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" / "))
            .unwrap_or_default();
        let casts = item["casts"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" / "))
            .unwrap_or_default();
        let cover = item["cover"].as_str().unwrap_or("");
        let link = item["url"].as_str().unwrap_or("");

        let description = format!(
            "标题：{}<br>评分：{}<br>导演：{}<br>主演：{}<br><img src=\"{}\" referrerpolicy=\"no-referrer\">",
            title, rate, directors, casts, cover
        );

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(link.to_string())
            .description(description)
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(title)
        .link("https://movie.douban.com/tag/#/?sort=U&range=0,10&tags=")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
