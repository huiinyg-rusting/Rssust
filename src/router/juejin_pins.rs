use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let type_param = para.get("type").map(|s| s.as_str()).unwrap_or("recommend");

    let title_label = match type_param {
        "recommend" => "推荐",
        "hot" => "热门",
        "6824710203301167112" => "上班摸鱼",
        "6819970850532360206" => "内推招聘",
        "6824710202487472141" => "一图胜千言",
        "6824710202562969614" => "今天学到了",
        "6824710202378436621" => "每天一道算法题",
        "6824710202000932877" => "开发工具推荐",
        "6824710203112423437" => "树洞一下",
        _ => "推荐",
    };

    let (url, body) = if type_param.chars().all(|c| c.is_ascii_digit()) {
        (
            "https://api.juejin.cn/recommend_api/v1/short_msg/topic".to_string(),
            format!(
                r#"{{"id_type":4,"sort_type":500,"cursor":"0","limit":20,"topic_id":"{}"}}"#,
                type_param
            ),
        )
    } else {
        (
            format!(
                "https://api.juejin.cn/recommend_api/v1/short_msg/{}",
                type_param
            ),
            r#"{"id_type":4,"sort_type":200,"cursor":"0","limit":20}"#.to_string(),
        )
    };

    let resp = fetch_reqwest_post_json(&url, &body)?;
    let json: Value = serde_json::from_str(&resp)?;

    let items_data = json
        .pointer("/data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data 字段"))?;

    let mut item_vec = Vec::new();
    for item in items_data {
        let msg_info = item
            .get("msg_Info")
            .ok_or_else(|| anyhow!("缺少 msg_Info"))?;
        let content = msg_info["content"].as_str().unwrap_or("").to_string();
        let ctime = msg_info["ctime"]
            .as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| msg_info["ctime"].as_i64())
            .unwrap_or(0);
        let msg_id = item["msg_id"].as_str().unwrap_or("");
        let author = item["author_user_info"]["user_name"]
            .as_str()
            .unwrap_or("");

        let link = format!("https://juejin.cn/pin/{}", msg_id);
        let pub_date = timestamp_to_rss(ctime);

        let mut description = content.replace('\n', "<br>");
        if let Some(pic_list) = msg_info["pic_list"].as_array() {
            for pic in pic_list {
                if let Some(src) = pic.as_str() {
                    description.push_str(&format!("<br><img src=\"{}\">", src));
                }
            }
        }

        let rss_item = ItemBuilder::default()
            .title(Some(content.clone()))
            .link(link)
            .description(Some(description))
            .pub_date(pub_date)
            .author(Some(author.to_string()))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("掘金沸点 - {}", title_label))
        .link("https://juejin.cn/pins/recommended")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
