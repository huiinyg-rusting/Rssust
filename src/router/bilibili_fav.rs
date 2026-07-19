use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let uid = para
        .get("uid")
        .ok_or_else(|| anyhow!("缺少 uid 参数"))?;
    let fid = para
        .get("fid")
        .ok_or_else(|| anyhow!("缺少 fid 参数"))?;

    let url = format!(
        "https://api.bilibili.com/x/v3/fav/resource/list?media_id={}&ps=20",
        fid
    );
    let referer = format!("https://space.bilibili.com/{}/", uid);

    let cookie = load_cookie_header(Some("bilibili.com")).ok().flatten();
    let mut headers: Vec<(&str, &str)> = vec![("Referer", referer.as_str())];
    if let Some(c) = &cookie {
        headers.push(("Cookie", c));
    }

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &headers)?.as_str(),
    )?;

    let code = json["code"].as_i64().unwrap_or(-1);
    if code != 0 {
        return Err(anyhow!("API 返回错误: {:?}", json["message"]));
    }

    let username = json
        .pointer("/data/info/upper/name")
        .and_then(Value::as_str)
        .unwrap_or("UP主");
    let fav_name = json
        .pointer("/data/info/title")
        .and_then(Value::as_str)
        .unwrap_or("收藏夹");

    let medias = json
        .pointer("/data/medias")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data/medias 字段或不是数组"))?;

    let mut item_vec = Vec::new();

    for item in medias {
        let id = item["id"].as_i64().unwrap_or(0);
        let bvid = item["bvid"].as_str().unwrap_or_default();
        let cover = item["cover"].as_str().unwrap_or_default();
        let intro = item["intro"].as_str().unwrap_or_default();
        let fav_time = item["fav_time"].as_i64().unwrap_or(0);
        let item_author = item.pointer("/upper/name").and_then(Value::as_str).unwrap_or("");

        let description = match para.get("disableembed") {
            Some(s) if s == "false" => "".to_owned(),
            _ => {
                format!(
                    r#"<iframe width="640" height="360" src="https://www.bilibili.com/blackboard/html5mobileplayer.html?aid={}&amp;cid=undefined&amp;bvid={}" frameborder="0" allowfullscreen="" referrerpolicy="no-referrer"></iframe><br><img src="{}" referrerpolicy="no-referrer"><br>{}"#,
                    id, bvid, cover, intro
                )
            }
        };

        let pub_date = timestamp_to_rss(fav_time);

        let rss_item = ItemBuilder::default()
            .title(Some(no_double_quotes(item["title"].to_string())))
            .link(Some(format!("https://www.bilibili.com/video/{}", bvid)))
            .description(description)
            .pub_date(pub_date)
            .author(Some(item_author.to_string()))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("{} 的 bilibili 收藏夹 {}", username, fav_name))
        .link(format!(
            "https://space.bilibili.com/{}/#/favlist?fid={}",
            uid, fid
        ))
        .description(format!("{} 的 bilibili 收藏夹 {}", username, fav_name))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
