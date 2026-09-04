use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::NaiveDate;
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const MAINTAINER: &str = "AI制作 / huiinyg-rusting审核";

fn daily_pubdate(date_str: &str) -> Option<String> {
    let d = NaiveDate::parse_from_str(date_str, "%Y%m%d").ok()?;
    Some(d.format("%a, %d %b %Y 00:00:00 +0800").to_string())
}

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let date = para.get("date").map(|s| s.as_str()).unwrap_or("");
    let api = if date.is_empty() {
        "https://news-at.zhihu.com/api/4/news/latest".to_string()
    } else {
        format!("https://news-at.zhihu.com/api/4/news/before/{}", date)
    };

    let json: Value = serde_json::from_str(fetch_reqwest_get(&api).await?.as_str())?;
    let date_field = json["date"].as_str().unwrap_or(date).to_string();
    let stories = json["stories"]
        .as_array()
        .ok_or_else(|| anyhow!("知乎日报缺少 stories"))?;

    let mut item_vec = Vec::new();
    for s in stories {
        let title = s["title"].as_str().unwrap_or("").to_string();
        if title.is_empty() {
            continue;
        }
        let link = s["url"]
            .as_str()
            .map(|u| u.to_string())
            .unwrap_or_default();
        let img = s["images"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let mut desc = String::new();
        if !img.is_empty() {
            desc.push_str(&format!(
                "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>",
                img
            ));
        }
        if let Some(hint) = s["hint"].as_str()
            && !hint.is_empty()
        {
            desc.push_str(&format!("<p>{}</p>", hint));
        }

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(link.clone())
                .guid(Some(
                    GuidBuilder::default()
                        .value(link.clone())
                        .permalink(true)
                        .build(),
                ))
                .description(Some(desc))
                .pub_date(if date_field.is_empty() {
                    now()
                } else {
                    daily_pubdate(&date_field).unwrap_or_else(now)
                })
                .build(),
        );
    }

    let channel = ChannelBuilder::default()
        .title(format!(
            "知乎日报{}",
            if date_field.is_empty() {
                String::new()
            } else {
                format!(" {}", date_field)
            }
        ))
        .link("https://daily.zhihu.com")
        .description(format!("知乎日报官方 API 聚合 | {}", MAINTAINER))
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}