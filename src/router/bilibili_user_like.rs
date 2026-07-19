use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let uid = para
        .get("uid")
        .ok_or_else(|| anyhow!("缺少 uid 参数"))?;

    let url = format!(
        "https://api.bilibili.com/x/space/like/video?vmid={}",
        uid
    );
    let referer = format!("https://space.bilibili.com/{}/", uid);
    let headers: Vec<(&str, &str)> = vec![("Referer", referer.as_str())];

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &headers)?.as_str(),
    )?;

    let code = json["code"].as_i64().unwrap_or(-1);
    if code != 0 {
        return Err(anyhow!("API 返回错误: {:?}", json["message"]));
    }

    let list = json
        .pointer("/data/list")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data/list 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    let mut author = String::from("UP主");

    for item in list {
        let aid = item["aid"].as_i64().unwrap_or(0);
        let bvid = item["bvid"].as_str().unwrap_or_default();
        let pic = item["pic"].as_str().unwrap_or_default();
        let desc = item["desc"].as_str().unwrap_or_default();
        let pubdate = item["pubdate"].as_i64().unwrap_or(0);

        if let Some(name) = item.pointer("/owner/name").and_then(Value::as_str) {
            if author == "UP主" {
                author = name.to_string();
            }
        }

        let description = match para.get("disableembed") {
            Some(s) if s == "false" => "".to_owned(),
            _ => {
                format!(
                    r#"<iframe width="640" height="360" src="https://www.bilibili.com/blackboard/html5mobileplayer.html?aid={}&amp;cid=undefined&amp;bvid={}" frameborder="0" allowfullscreen="" referrerpolicy="no-referrer"></iframe><br><img src="{}" referrerpolicy="no-referrer"><br>{}"#,
                    aid, bvid, pic, desc
                )
            }
        };

        let pub_date = timestamp_to_rss(pubdate);

        let rss_item = ItemBuilder::default()
            .title(Some(no_double_quotes(item["title"].to_string())))
            .link(Some(format!("https://www.bilibili.com/video/{}", bvid)))
            .description(description)
            .pub_date(pub_date)
            .author(author.clone())
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("{} 的 bilibili 点赞视频", author))
        .link(format!("https://space.bilibili.com/{}", uid))
        .description(format!("{} 的 bilibili 点赞视频", author))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
