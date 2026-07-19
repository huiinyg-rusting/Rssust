use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let tid = para
        .get("tid")
        .ok_or_else(|| anyhow!("缺少 tid 参数"))?;

    let url = format!(
        "https://api.bilibili.com/x/web-interface/newlist?ps=15&rid={}&_={}",
        tid,
        chrono::Local::now().timestamp_millis()
    );
    let headers: Vec<(&str, &str)> = vec![("Referer", "https://www.bilibili.com/")];

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &headers)?.as_str(),
    )?;

    let list = json
        .pointer("/data/archives")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data/archives 字段或不是数组"))?;

    let name = list
        .first()
        .and_then(|v| v["tname"].as_str())
        .unwrap_or("未知");

    let mut item_vec = Vec::new();

    for video in list {
        let aid = video["aid"].as_i64().unwrap_or(0);
        let bvid = video["bvid"].as_str().unwrap_or_default();
        let pic = video["pic"].as_str().unwrap_or_default();
        let desc = video["desc"].as_str().unwrap_or_default();
        let pubdate = video["pubdate"].as_i64().unwrap_or(0);
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

        let pub_date = timestamp_to_rss(pubdate);

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(video["title"].to_string())))
            .link(Some(format!("https://www.bilibili.com/video/{}", bvid)))
            .description(description)
            .pub_date(pub_date)
            .author(Some(author.to_string()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("bilibili {}分区", name))
        .link("https://www.bilibili.com")
        .description(format!("bilibili {}分区", name))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
