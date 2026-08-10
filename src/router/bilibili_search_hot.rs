use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let json: Value = serde_json::from_str(
        fetch_reqwest_get("https://s.search.bilibili.com/main/hotword")
            .await?
            .as_str(),
    )?;

    if json.pointer("/code").and_then(Value::as_i64) != Some(0) {
        return Err(anyhow!(
            "B站API返回错误 code={}: {}",
            json.pointer("/code").and_then(Value::as_i64).unwrap_or(-1),
            json.pointer("/message")
                .and_then(Value::as_str)
                .unwrap_or("")
        ));
    }

    let list = json
        .pointer("/list")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("找不到 list 字段或不是数组"))?;

    let timestamp = json
        .pointer("/timestamp")
        .and_then(Value::as_i64)
        .unwrap_or(0);

    let mut item_vec = Vec::new();

    for word in list {
        let keyword = word["keyword"].as_str().unwrap_or_default();
        let show_name = word["show_name"].as_str().unwrap_or_default();
        let pos = word["pos"].as_i64().unwrap_or(0);
        let hot_id = word["hot_id"].as_i64().unwrap_or(0);
        let word_type = word["word_type"].as_i64().unwrap_or(0);
        let icon = word["icon"].as_str().unwrap_or_default();

        let title = if show_name.is_empty() {
            format!("#{} {}", pos, keyword)
        } else {
            format!("#{} {}", pos, show_name)
        };

        let type_name = match word_type {
            4 => "新",
            5 => "热",
            7 => "直播中",
            9 => "梗",
            11 => "话题",
            12 => "独家",
            _ => "普通",
        };

        let link = format!(
            "https://search.bilibili.com/all?keyword={}",
            urlencoding::encode(keyword)
        );

        let description = if icon.is_empty() {
            format!("热搜词: {}<br>类型: {}<br>热词id: {}", show_name, type_name, hot_id)
        } else {
            format!(
                "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>热搜词: {}<br>类型: {}<br>热词id: {}",
                icon, show_name, type_name, hot_id
            )
        };

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(title)))
            .link(Some(link))
            .description(description)
            .pub_date(timestamp_to_rss(timestamp))
            .author(Some(type_name.to_string()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title("B站热搜榜".to_string())
        .link("https://www.bilibili.com/".to_string())
        .description("B站热搜榜单前10关键词".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
