use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let list_type = para
        .get("list_type")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "1".to_string());

    let url = format!(
        "https://api.bilibili.com/x/copyright-music-publicity/toplist/all_period?list_type={}",
        list_type
    );
    let json: Value = serde_json::from_str(fetch_reqwest_get(&url).await?.as_str())?;

    if json.pointer("/code").and_then(Value::as_i64) != Some(0) {
        return Err(anyhow!(
            "B站API返回错误 code={}: {}",
            json.pointer("/code").and_then(Value::as_i64).unwrap_or(-1),
            json.pointer("/message")
                .and_then(Value::as_str)
                .unwrap_or("")
        ));
    }

    let list_obj = json
        .pointer("/data/list")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("找不到 data.list 字段或不是对象"))?;

    let mut latest_id: i64 = 0;
    let mut latest_priod: i64 = 0;
    let mut latest_time: i64 = 0;
    let mut latest_year: i32 = -1;

    for (year, periods) in list_obj {
        if let Ok(y) = year.parse::<i32>() {
            if let Some(arr) = periods.as_array() {
                for p in arr {
                    let t = p["publish_time"].as_i64().unwrap_or(0);
                    if y > latest_year || t > latest_time {
                        latest_year = y;
                        latest_time = t;
                        latest_id = p["ID"].as_i64().unwrap_or(0);
                        latest_priod = p["priod"].as_i64().unwrap_or(0);
                    }
                }
            }
        }
    }

    if latest_id == 0 {
        return Err(anyhow!("找不到音频榜单期数"));
    }

    let rank_name = if list_type == "2" { "原创榜" } else { "热榜" };

    let url = format!(
        "https://api.bilibili.com/x/copyright-music-publicity/toplist/music_list?list_id={}",
        latest_id
    );
    let json: Value = serde_json::from_str(fetch_reqwest_get(&url).await?.as_str())?;

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
        .ok_or_else(|| anyhow!("找不到 data.list 字段或不是数组"))?;

    let mut item_vec = Vec::new();

    for song in list {
        let rank = song["rank"].as_i64().unwrap_or(0);
        let music_title = song["music_title"].as_str().unwrap_or_default();
        let singer = song["singer"].as_str().unwrap_or_default();
        let album = song["album"].as_str().unwrap_or_default();
        let heat = song["heat"].as_i64().unwrap_or(0);
        let can_listen = song["can_listen"].as_bool().unwrap_or(false);
        let mv_bvid = song["mv_bvid"].as_str().unwrap_or_default();
        let creation_bvid = song["creation_bvid"].as_str().unwrap_or_default();
        let creation_title = song["creation_title"].as_str().unwrap_or_default();
        let creation_nickname = song["creation_nickname"].as_str().unwrap_or_default();
        let creation_cover = song["creation_cover"].as_str().unwrap_or_default();

        let title = format!("#{} {} - {}", rank, music_title, singer);

        let link = if !mv_bvid.is_empty() {
            format!("https://www.bilibili.com/video/{}", mv_bvid)
        } else if !creation_bvid.is_empty() {
            format!("https://www.bilibili.com/video/{}", creation_bvid)
        } else {
            format!("https://music.bilibili.com/pc/rank?list_id={}", latest_id)
        };

        let creation_link = if creation_bvid.is_empty() {
            String::new()
        } else {
            format!(
                "<br>关联视频: <a href=\"https://www.bilibili.com/video/{}\">{}</a>（{}）",
                creation_bvid, creation_title, creation_nickname
            )
        };

        let description = format!(
            "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>歌手: {}<br>专辑: {}<br>热度: {}<br>可听: {}{}",
            creation_cover,
            singer,
            album,
            heat,
            if can_listen { "是" } else { "否" },
            creation_link
        );

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(title)))
            .link(Some(link))
            .description(description)
            .pub_date(timestamp_to_rss(latest_time))
            .author(Some(singer.to_string()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("B站音频{} - 第{}期", rank_name, latest_priod))
        .link(format!(
            "https://music.bilibili.com/pc/rank?list_id={}",
            latest_id
        ))
        .description(format!(
            "B站音频{}最新一期榜单（共{}首）",
            rank_name,
            list.len()
        ))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
