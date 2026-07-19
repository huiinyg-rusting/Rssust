use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

fn get_aid_from_bvid(bvid: &str) -> Result<i64, Error> {
    let url = format!(
        "https://api.bilibili.com/x/web-interface/view?bvid={}",
        bvid
    );
    let json: Value = serde_json::from_str(fetch_reqwest_get(&url)?.as_str())?;
    json.pointer("/data/aid")
        .and_then(Value::as_i64)
        .ok_or_else(|| anyhow!("无法从 BVID 获取 AID"))
}

fn get_video_name(bvid: &str, aid: Option<i64>) -> String {
    let url = if let Some(a) = aid {
        format!("https://api.bilibili.com/x/web-interface/view?aid={}", a)
    } else {
        format!(
            "https://api.bilibili.com/x/web-interface/view?bvid={}",
            bvid
        )
    };
    fetch_reqwest_get(&url)
        .ok()
        .and_then(|s| {
            serde_json::from_str::<Value>(&s)
                .ok()
                .and_then(|v| {
                    v.pointer("/data/title")
                        .and_then(Value::as_str)
                        .map(|t| t.to_string())
                })
        })
        .unwrap_or_else(|| bvid.to_string())
}

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let bvid = para
        .get("bvid")
        .ok_or_else(|| anyhow!("缺少 bvid 参数"))?;

    let aid = get_aid_from_bvid(bvid)?;
    let name = get_video_name(bvid, Some(aid));

    let link = format!("https://www.bilibili.com/video/{}", bvid);
    let url = format!(
        "https://api.bilibili.com/x/v2/reply?type=1&oid={}&sort=0",
        aid
    );
    let cookie = load_cookie_header(Some("bilibili.com")).ok().flatten().unwrap_or_default();

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(
            &url,
            &[
                ("Referer", link.as_str()),
                ("Cookie", cookie.as_str()),
            ],
        )?.as_str(),
    )?;

    let replies = json
        .pointer("/data/replies")
        .and_then(|v| v.as_array())
        .map(|v| v.clone())
        .unwrap_or_default();

    let mut item_vec = Vec::new();

    for reply in replies {
        let uname = reply
            .pointer("/member/uname")
            .and_then(Value::as_str)
            .unwrap_or("未知用户");
        let message = reply
            .pointer("/content/message")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let ctime = reply["ctime"].as_i64().unwrap_or(0);
        let rpid = reply["rpid"].as_i64().unwrap_or(0);

        let title = format!("{} : {}", uname, message);
        let description = title.clone();
        let pub_date = timestamp_to_rss(ctime);
        let reply_link = format!("{}/#reply{}", link, rpid);

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(reply_link)
            .description(description)
            .pub_date(pub_date)
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("{} 的 评论", name))
        .link(link)
        .description(format!("{} 的评论", name))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
