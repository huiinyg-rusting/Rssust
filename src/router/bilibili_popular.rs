use crate::easyuser::*;
use anyhow::{Error, Ok, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let pn = para.get("pn").map(|s| s.as_str()).unwrap_or("1");
    let ps = para.get("ps").map(|s| s.as_str()).unwrap_or("30");
    let json: Value = serde_json::from_str(
        fetch_reqwest_get(
            format!(
                "https://api.bilibili.com/x/web-interface/popular?pn={}&ps={}",
                pn, ps
            )
            .as_str(),
        )?
        .as_str(),
    )?;

    let mut item_vec = Vec::new();
    let list = json
        .pointer("/data/list")
        .and_then(|list| list.as_array())
        .ok_or_else(|| anyhow!("找不到 data.list 字段或不是数组"))?;

    for video in list {
        let aid = video["aid"]
            .as_u64()
            .ok_or_else(|| anyhow!("缺少 aid 字段"))?;
        let bvid = video["bvid"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 bvid 字段"))?;
        let pic = video["pic"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 pic 字段"))?;
        let desc = video["desc"].as_str().unwrap_or_default();
        let pubdate = video["pubdate"]
            .as_i64()
            .ok_or_else(|| anyhow!("缺少 pubdate 字段"))?;
        let author = video["owner"]["name"].as_str().unwrap_or("未知");

        let description = match para.get("disableembed") {
            Some(s) if s == "false" => "".to_owned(),
            _ => {
                format!(
                    r#"<iframe width="640" height="360" src="https://www.bilibili.com/blackboard/html5mobileplayer.html?aid={}&amp;cid=undefined&amp;bvid={}" frameborder="0" allowfullscreen="" referrerpolicy="no-referrer"></iframe><br><img src="{}" referrerpolicy="no-referrer"><br>{}"#,
                    aid, bvid, pic, desc
                )
            }
        };

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(video["title"].to_string())))
            .link(Some(format!("https://www.bilibili.com/video/{}", bvid)))
            .description(description)
            .pub_date(timestamp_to_rss(pubdate))
            .author(Some(author.to_owned()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title("B站热门视频")
        .link("https://www.bilibili.com/v/popular/rank/all")
        .description("B站热门视频精选")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
