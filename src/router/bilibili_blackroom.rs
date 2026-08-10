use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let pn = para
        .get("pn")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "1".to_string());
    let btype = para.get("btype").cloned().unwrap_or_default();
    let otype = para
        .get("otype")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "0".to_string());

    let url = format!(
        "https://api.bilibili.com/x/credit/blocked/list?btype={}&otype={}&pn={}",
        btype, otype, pn
    );

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &[("Referer", "https://www.bilibili.com/blackroom/")])
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

    let data = json
        .pointer("/data")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("找不到 data 字段或不是数组"))?;

    let mut item_vec = Vec::new();

    for item in data {
        let uname = item["uname"].as_str().unwrap_or_default();
        let uid = item["uid"].as_i64().unwrap_or(0);
        let face = item["face"].as_str().unwrap_or_default();
        let punish_title = item["punishTitle"].as_str().unwrap_or_default();
        let punish_type = item["punishTypeName"].as_str().unwrap_or_default();
        let reason = item["reasonTypeName"].as_str().unwrap_or_default();
        let origin_title = item["originTitle"].as_str().unwrap_or_default();
        let blocked_days = item["blockedDays"].as_i64().unwrap_or(0);
        let punish_time = item["punishTime"].as_i64().unwrap_or(0);
        let comment_sum = item["commentSum"].as_i64().unwrap_or(0);
        let origin_type = item["originTypeName"].as_str().unwrap_or_default();

        let title = if punish_title.is_empty() {
            format!("{}({})", uname, uid)
        } else {
            format!("{} - {}", punish_title, uname)
        };

        let description = format!(
            "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>用户: <a href=\"https://space.bilibili.com/{}/\">{}</a><br>来源: {}<br>封禁原因: {}<br>处理手段: {}<br>封禁天数: {}<br>来源标题: {}<br>评论数: {}",
            face,
            uid,
            uname,
            origin_type,
            reason,
            punish_type,
            blocked_days,
            origin_title,
            comment_sum
        );

        let link = format!(
            "https://www.bilibili.com/blackroom/ban#/detail/{}",
            item["id"]
        );

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(Some(link))
            .description(description)
            .pub_date(timestamp_to_rss(punish_time))
            .author(Some(uname.to_string()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title("B站小黑屋封禁公示".to_string())
        .link("https://www.bilibili.com/blackroom/".to_string())
        .description("B站小黑屋封禁公示列表".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
