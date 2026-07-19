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
        "https://api.bilibili.com/x/v2/fav/video?vmid={}&ps=30&tid=0&keyword=&pn=1&order=fav_time",
        uid
    );
    let referer = format!("https://space.bilibili.com/{}/#/favlist", uid);

    let cookie = load_cookie_header(Some("bilibili.com")).ok().flatten();
    let mut headers: Vec<(&str, &str)> = vec![("Referer", referer.as_str())];
    if let Some(c) = &cookie {
        headers.push(("Cookie", c));
    }

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &headers)?.as_str(),
    )?;

    let archives = json
        .pointer("/data/archives")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data/archives 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    let mut author = String::from("UP主");

    for item in archives {
        let aid = item["aid"].as_i64().unwrap_or(0);
        let bvid = item["bvid"].as_str().unwrap_or_default();
        let pic = item["pic"].as_str().unwrap_or_default();
        let desc = item["desc"].as_str().unwrap_or_default();
        let fav_at = item["fav_at"].as_i64().unwrap_or(0);

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

        let pub_date = timestamp_to_rss(fav_at);

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
        .title(format!("{} 的 bilibili 收藏夹", author))
        .link(format!("https://space.bilibili.com/{}/#/favlist", uid))
        .description(format!("{} 的 bilibili 收藏夹", author))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
