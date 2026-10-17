use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

/* 允许的频道：key 为对外参数值，中文名用于频道标题 */
const CHANNELS: &[(&str, &str, &str)] = &[
    ("global", "global-channel", "全部"),
    ("astock", "a-stock-channel", "A股"),
    ("hkstock", "hk-stock-channel", "港股"),
    ("usstock", "us-stock-channel", "美股"),
    ("forex", "forex-channel", "外汇"),
    ("oil", "oil-channel", "原油"),
    ("gold", "goldc-channel", "黄金"),
    ("commodity", "commodity-channel", "商品"),
];

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    // 上游把空 title 的快讯也返回，正文放 content / content_text 里，标题需回退取正文首行
    let channel = req_optional(&para, "channel").unwrap_or_default();
    let (channel_id, channel_name) = if channel.is_empty() {
        ("global-channel", "全部")
    } else {
        CHANNELS
            .iter()
            .find(|(key, _, _)| *key == channel)
            .map(|(_, id, name)| (*id, *name))
            .ok_or_else(|| {
                let keys: Vec<&str> = CHANNELS.iter().map(|(k, _, _)| *k).collect();
                anyhow!(
                    "未知 channel \"{}\"，可选值：{}，留空表示全部快讯",
                    channel,
                    keys.join(" / ")
                )
            })?
    };

    // 上游 limit>100 只返回 100 条，故钳到 100
    let limit = opt_usize(&para, "limit", 20).clamp(1, 100);
    let url = format!(
        "https://api-one.wallstcn.com/apiv1/content/lives?channel={}&limit={}",
        channel_id, limit
    );

    let json: Value = serde_json::from_str(fetch_reqwest_get(&url).await?.as_str())?;
    let items = json
        .pointer("/data/items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data/items 字段或不是数组"))?;
    if items.is_empty() {
        return Err(anyhow!("华尔街见闻快讯返回空列表，channel \"{}\" 可能无效", channel));
    }

    let mut item_vec = Vec::new();
    for item in items {
        let link = item["uri"].as_str().unwrap_or("");
        let content = item["content"].as_str().unwrap_or("");
        let content_text = item["content_text"].as_str().unwrap_or("");
        let display_time = item["display_time"].as_i64().unwrap_or(0);

        // title 可能为空字符串，依次回退：highlight_title -> title -> content_text 首句
        let title = item["highlight_title"]
            .as_str()
            .filter(|s| !s.is_empty())
            .or_else(|| item["title"].as_str().filter(|s| !s.is_empty()))
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                let first = content_text
                    .lines()
                    .map(str::trim)
                    .find(|l| !l.is_empty())
                    .unwrap_or("7x24快讯");
                let mut t: String = first.chars().take(50).collect();
                if first.chars().count() > 50 {
                    t.push('…');
                }
                t
            });

        let mut description = if content.is_empty() {
            content_text.to_string()
        } else {
            content.to_string()
        };
        // 正文为空时用标题兜底，避免 description 为空导致条目不可见
        if description.is_empty() {
            description = title.clone();
        }

        let rss_item = ItemBuilder::default()
            .title(Some(title))
            .link(if link.is_empty() {
                "https://wallstreetcn.com/live".to_string()
            } else {
                link.to_string()
            })
            .pub_date(timestamp_to_rss(display_time))
            .description(Some(description))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("华尔街见闻快讯 - {}", channel_name))
        .link("https://wallstreetcn.com/live")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
