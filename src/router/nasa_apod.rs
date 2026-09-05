use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const MAINTAINER: &str = "AI制作 / huiinyg-rusting审核";

fn apod_pubdate(date_str: &str) -> Option<String> {
    date_str_to_rss(date_str, "%Y-%m-%d", "+0000")
}

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let count = para
        .get("count")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(5)
        .clamp(1, 30);
    let api = format!(
        "https://api.nasa.gov/planetary/apod?api_key=DEMO_KEY&count={}",
        count
    );

    let json: Value = serde_json::from_str(fetch_reqwest_get(&api).await?.as_str())?;
    let arr = json
        .as_array()
        .ok_or_else(|| anyhow!("NASA APOD 返回不是数组"))?;

    let mut item_vec = Vec::new();
    for d in arr {
        let title = d["title"].as_str().unwrap_or("NASA APOD");
        let date = d["date"].as_str().unwrap_or("");
        let media_type = d["media_type"].as_str().unwrap_or("image");
        let media = d["hdurl"]
            .as_str()
            .unwrap_or(d["url"].as_str().unwrap_or(""))
            .to_string();
        let explanation = d["explanation"].as_str().unwrap_or("");
        let copyright = d["copyright"].as_str().unwrap_or("");

        let mut desc = String::new();
        if media_type == "image" && !media.is_empty() {
            desc.push_str(&format!(
                "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>",
                media
            ));
        }
        desc.push_str(&format!("<p>{}</p>", explanation));
        if !copyright.is_empty() {
            desc.push_str(&format!("<p><em>© {}</em></p>", copyright));
        }

        let final_title = if date.is_empty() {
            title.to_string()
        } else {
            format!("{} ({})", title, date)
        };
        item_vec.push(
            ItemBuilder::default()
                .title(Some(final_title))
                .link(media.clone())
                .guid(Some(
                    GuidBuilder::default()
                        .value(media.clone())
                        .permalink(true)
                        .build(),
                ))
                .description(Some(desc))
                .pub_date(if date.is_empty() {
                    now()
                } else {
                    apod_pubdate(date).unwrap_or_else(now)
                })
                .build(),
        );
    }

    let channel = ChannelBuilder::default()
        .title("NASA Astronomy Picture of the Day")
        .link("https://apod.nasa.gov/apod/astropix.html")
        .description(format!("NASA 每日天文一图 | {}", MAINTAINER))
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}