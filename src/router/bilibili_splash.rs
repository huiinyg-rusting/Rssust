use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let build = para
        .get("build")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "999999999".to_string());
    let mobi_app = para
        .get("mobi_app")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "android".to_string());
    let platform = para
        .get("platform")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "android".to_string());
    let height = para
        .get("height")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "1920".to_string());
    let width = para
        .get("width")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "1080".to_string());
    let birth = para
        .get("birth")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "0101".to_string());

    let url = format!(
        "https://app.bilibili.com/x/v2/splash/list?build={}&mobi_app={}&platform={}&height={}&width={}&birth={}",
        build, mobi_app, platform, height, width, birth
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
        .cloned()
        .unwrap_or_default();

    let mut item_vec = Vec::new();

    for item in list {
        let id = item["id"].as_i64().unwrap_or(0);
        let thumb = item["thumb"].as_str().unwrap_or_default();
        let uri = item["uri"].as_str().unwrap_or_default();
        let video_url = item["video_url"].as_str().unwrap_or_default();
        let is_ad = item["is_ad"].as_bool().unwrap_or(false);
        let duration = item["duration"].as_i64().unwrap_or(0);
        let begin_time = item["begin_time"].as_i64().unwrap_or(0);
        let _end_time = item["end_time"].as_i64().unwrap_or(0);

        let title = item["uri_title"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or(if is_ad {
                "B站开屏广告"
            } else {
                "B站开屏图片"
            });

        let mut description = format!(
            "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>广告id: {}<br>是否广告: {}<br>时长(秒): {}",
            thumb, id, is_ad, duration
        );
        if !uri.is_empty() {
            description.push_str(&format!("<br>跳转链接: <a href=\"{}\">{}</a>", uri, uri));
        }
        if !video_url.is_empty() {
            description.push_str(&format!(
                "<br>视频: <a href=\"{}\">{}</a>",
                video_url, video_url
            ));
        }

        let item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(Some(if uri.is_empty() {
                thumb.to_string()
            } else {
                uri.to_string()
            }))
            .description(description)
            .pub_date(timestamp_to_rss(begin_time))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title("B站APP端开屏广告".to_string())
        .link("https://app.bilibili.com".to_string())
        .description("B站APP端开屏广告信息".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
