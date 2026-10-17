use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let id = opt_usize(&para, "id", 3778678);

    let json: Value = serde_json::from_str(
        fetch_reqwest_get(&format!(
            "https://music.163.com/api/playlist/detail?id={}",
            id
        ))
        .await?
        .as_str(),
    )?;

    // 老接口偶尔会抽风返回 code=-460 提示"网络太拥挤"，直接当错误抛给上层
    if json["code"].as_i64() != Some(200) {
        return Err(anyhow!(
            "网易云API返回错误 code={}",
            json["code"].as_i64().unwrap_or(-1)
        ));
    }

    let result = json["result"]
        .as_object()
        .ok_or_else(|| anyhow!("找不到 result 字段或不是对象"))?;
    let name = result
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("热歌榜");
    let update_time = result
        .get("updateTime")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        / 1000; // 接口返回的是毫秒，RSS 日期要秒
    let tracks = result
        .get("tracks")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("找不到 tracks 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    for t in tracks {
        let title = t["name"].as_str().unwrap_or_default();
        let tid = t["id"].as_i64().unwrap_or(0);
        if title.is_empty() || tid == 0 {
            continue;
        }

        let artists: Vec<&str> = t["artists"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|a| a["name"].as_str()).collect())
            .unwrap_or_default();
        let album = t["album"]["name"].as_str().unwrap_or_default();

        let mut description = String::new();
        if !artists.is_empty() {
            description.push_str(&format!("歌手: {}<br>", artists.join(" / ")));
        }
        if !album.is_empty() {
            description.push_str(&format!("专辑: {}", album));
        }

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(title.to_string())))
            .link(Some(format!("https://music.163.com/song?id={}", tid)))
            .description(Some(description))
            .pub_date(if update_time > 0 {
                timestamp_to_rss(update_time)
            } else {
                now()
            })
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("网易云音乐 - {}", name))
        .link(format!("https://music.163.com/discover/toplist?id={}", id))
        .description("网易云音乐排行榜".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
