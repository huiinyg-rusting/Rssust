use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

///麻省理工科技评论 (MIT Technology Review 中文站)
///type: index=首页资讯 hot=本周热榜 breaking=快讯 video=视频，默认 index
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let route_type = para.get("type").cloned().unwrap_or_else(|| "index".to_string());
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10)
        .min(30);

    let (title, api_path) = match route_type.as_str() {
        "breaking" => ("快讯", "/flash"),
        "hot" => ("本周热榜", "/information/hot"),
        "video" => ("视频", "/movie/index"),
        _ => ("首页资讯", "/information/index"),
    };

    let api = format!("https://apii.web.mittrchina.com{}", api_path);
    let resp = if route_type == "breaking" {
        fetch_reqwest_post(
            &api,
            format!("page=1&size={}", limit),
            None,
        )
        .await?
    } else {
        fetch_reqwest_get_with_headers(
            &format!("{}?limit={}", api, limit),
            &[("User-Agent", UA)],
        )
        .await?
    };

    let json: Value = serde_json::from_str(&resp)?;
    let articles = if route_type == "hot" {
        json["data"].as_array().cloned().unwrap_or_default()
    } else {
        json["data"]["items"].as_array().cloned().unwrap_or_default()
    };

    let mut item_vec = Vec::new();
    for article in articles {
        let id = article["id"].as_i64().unwrap_or(0);
        let is_video = route_type == "video";
        let name = if is_video {
            article["title"].as_str().unwrap_or("").to_string()
        } else {
            article["name"].as_str().unwrap_or("").to_string()
        };
        if name.is_empty() {
            continue;
        }
        let link = format!("https://www.mittrchina.com/news/detail/{}", id);

        let author = article["authors"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|a| a["username"].as_str())
            .or_else(|| {
                if is_video {
                    article["author"]
                        .as_array()
                        .and_then(|a| a.first())
                        .and_then(|a| a["username"].as_str())
                } else {
                    None
                }
            })
            .unwrap_or("")
            .to_string();

        let pub_date = article["start_time"]
            .as_i64()
            .or_else(|| article["push_time"].as_i64())
            .map(timestamp_to_rss)
            .unwrap_or_else(now);

        let mut description = if is_video {
            let addr = article["address"].as_str().unwrap_or("");
            if addr.is_empty() {
                article["summary"].as_str().unwrap_or("").to_string()
            } else {
                let ext = addr.rsplit('.').next().unwrap_or("mp4");
                format!(
                    "<video poster=\"{}\" controls><source src=\"{}\" type=\"video/{}\"></video>",
                    article["img"].as_str().unwrap_or(""),
                    addr,
                    ext
                )
            }
        } else if route_type == "breaking" {
            article["content"].as_str().unwrap_or("").to_string()
        } else {
            article["summary"].as_str().unwrap_or("").to_string()
        };

        // 非 breaking/video 类型抓详情页补全正文
        if !is_video && route_type != "breaking" && !description.is_empty() {
            if let Ok(detail) = fetch_reqwest_get_with_headers(
                &format!("https://apii.web.mittrchina.com/information/details?id={}", id),
                &[("User-Agent", UA)],
            )
            .await
            {
                if let Ok(detail_json) = serde_json::from_str::<Value>(&detail) {
                    if let Some(content) = detail_json["data"]["content"].as_str() {
                        if !content.is_empty() {
                            description = content.to_string();
                        }
                    }
                }
            }
        }

        let item = if !author.is_empty() {
            ItemBuilder::default()
                .title(Some(name))
                .link(link)
                .description(Some(description))
                .pub_date(pub_date)
                .author(Some(author))
                .build()
        } else {
            ItemBuilder::default()
                .title(Some(name))
                .link(link)
                .description(Some(description))
                .pub_date(pub_date)
                .build()
        };
        item_vec.push(item);
    }

    if item_vec.is_empty() {
        return Err(anyhow!("没有抓取到任何内容"));
    }

    let channel = ChannelBuilder::default()
        .title(format!("MIT 科技评论 - {}", title))
        .link(format!("https://www.mittrchina.com/{}", route_type))
        .description("麻省理工科技评论，发现改变世界的新兴科技")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
