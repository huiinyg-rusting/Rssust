use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const MAINTAINER: &str = "AI制作 / huiinyg-rusting审核";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const VALID_TABS: &[&str] = &["realtime", "game", "finance", "sport", "novel", "car"];

fn tab_title(tab: &str) -> String {
    match tab {
        "realtime" => "百度热搜".to_string(),
        _ => format!("百度热搜 - {}", tab),
    }
}

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let tab = para.get("tab").map(|s| s.as_str()).unwrap_or("realtime");
    if !VALID_TABS.contains(&tab) {
        return Err(anyhow!(HttpError::bad_request(format!(
            "不支持的 tab: {}，可用: {}",
            tab,
            VALID_TABS.join("/")
        ))));
    }
    let url = format!("https://top.baidu.com/board?tab={}", tab);

    let body = fetch_reqwest_get_with_headers(&url, &[("User-Agent", UA)]).await?;
    let re = Regex::new(r"<!--s-data:([\s\S]*?)-->").expect("baidu_top 正则编译失败");
    let caps = re
        .captures(&body)
        .ok_or_else(|| anyhow!(HttpError::bad_gateway("百度热搜页面未找到数据")))?;
    let json: Value = serde_json::from_str(caps.get(1).unwrap().as_str())?;
    let cards = json
        .pointer("/data/cards")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("百度热搜数据缺少 cards"))?;

    let mut item_vec = Vec::new();
    for card in cards {
        let content = match card["content"].as_array() {
            Some(c) => c,
            None => continue,
        };
        for it in content {
            let word = it["word"].as_str().unwrap_or("");
            if word.is_empty() {
                continue;
            }
            let link = it["rawUrl"]
                .as_str()
                .or_else(|| it["url"].as_str())
                .or_else(|| it["appUrl"].as_str())
                .unwrap_or("https://top.baidu.com/")
                .to_string();
            let desc = it["desc"].as_str().unwrap_or("");
            let hot = it["hotScore"].as_str().unwrap_or("");
            let img = it["img"].as_str().unwrap_or("");

            let mut desc_html = String::new();
            if !img.is_empty() {
                desc_html.push_str(&format!(
                    "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>",
                    img
                ));
            }
            desc_html.push_str(&format!("热度：{}<br><br>{}", hot, desc));

            item_vec.push(
                ItemBuilder::default()
                    .title(Some(word.to_string()))
                    .link(link)
                    .description(Some(desc_html))
                    .pub_date(now())
                    .build(),
            );
        }
    }

    let channel = ChannelBuilder::default()
        .title(tab_title(tab))
        .link(url)
        .description(format!("百度热搜榜单 | {}", MAINTAINER))
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}