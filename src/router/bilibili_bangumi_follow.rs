use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let vmid = para.get("vmid").ok_or_else(|| anyhow!("缺少 vmid 参数"))?;
    let btype = para
        .get("type")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "1".to_string());
    let pn = para
        .get("pn")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "1".to_string());
    let ps = para
        .get("ps")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "15".to_string());

    let url = format!(
        "https://api.bilibili.com/x/space/bangumi/follow/list?vmid={}&type={}&pn={}&ps={}",
        vmid, btype, pn, ps
    );

    let cookie = load_cookie_header(Some("bilibili.com")).ok().flatten();
    let referer = format!("https://space.bilibili.com/{}/bangumi", vmid);
    let mut headers: Vec<(&str, &str)> = vec![("Referer", &referer)];
    if let Some(c) = &cookie {
        headers.push(("Cookie", c));
    }

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &headers)
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
        .pointer("/data/list")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("找不到 data/list 字段或不是数组"))?;

    let mut item_vec = Vec::new();

    for item in list {
        let season_id = item["season_id"].as_i64().unwrap_or(0);
        let title = item["title"].as_str().unwrap_or_default();
        let cover = item["cover"].as_str().unwrap_or_default();
        let total_count = item["total_count"].as_i64().unwrap_or(-1);
        let is_finish = item["is_finish"].as_i64().unwrap_or(0);
        let evaluate = item["evaluate"].as_str().unwrap_or_default();
        let follows = item
            .pointer("/stat/follow")
            .and_then(Value::as_i64)
            .unwrap_or(0);

        let new_ep_index = item
            .pointer("/new_ep/index_show")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let new_ep_pub = item
            .pointer("/new_ep/pub_time")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let pub_date = if new_ep_pub.is_empty() {
            now()
        } else {
            datetime_str_to_rss(new_ep_pub).unwrap_or_else(now)
        };

        let description = format!(
            "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>总集数: {}<br>状态: {}<br>追番人数: {}<br>最新一话: {}<br>更新时间: {}<br><br>{}",
            cover,
            if total_count < 0 {
                "连载中".to_string()
            } else {
                total_count.to_string()
            },
            if is_finish == 1 {
                "已完结"
            } else {
                "连载中"
            },
            follows,
            if new_ep_index.is_empty() {
                "暂无"
            } else {
                new_ep_index
            },
            if new_ep_pub.is_empty() {
                "暂无"
            } else {
                new_ep_pub
            },
            evaluate
        );

        let item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(Some(format!(
                "https://www.bilibili.com/bangumi/play/ss{}?from_spmid=666.25.0.0",
                season_id
            )))
            .description(description)
            .pub_date(pub_date)
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("{} 的 bilibili 追番列表", vmid))
        .link(format!("https://space.bilibili.com/{}/bangumi", vmid))
        .description(format!("{} 的 bilibili 追番列表", vmid))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
