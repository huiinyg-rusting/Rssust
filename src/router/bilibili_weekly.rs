use crate::easyuser::*;
use anyhow::{Error, Ok, Result, anyhow};
use rss::*;
use serde_json::Value;

pub fn get() -> Result<String, Error> {
    let recommend = fetch_reqwest_get(
        "https://app.bilibili.com/x/v2/show/popular/selected/series?type=weekly_selected",
    )?;
    let recommend: Value = serde_json::from_str(recommend.as_str())?;
    let recommend = &recommend.pointer("/data/0/number").ok_or_else(|| {
        eprintln!("router.rs出现错误{}", recommend);
        anyhow!("router.rs出现错误{}", recommend)
    })?;
    let json: Value = serde_json::from_str(fetch_reqwest_get(format!("https://app.bilibili.com/x/v2/show/popular/selected?type=weekly_selected&number={}",recommend.to_string()).as_str())?.as_str())?;
    let mut item_vec = Vec::new();
    let list = json
        .pointer("/data/list")
        .and_then(|list| list.as_array())
        .ok_or_else(|| anyhow!("找不到 data.list 字段或不是数组"))?;

    for video in list {
        let description = format!(
            r#"<iframe width="640" height="360" src="https://www.bilibili.com/blackboard/html5mobileplayer.html?aid={}&amp;cid=undefined&amp;bvid={}" frameborder="0" allowfullscreen="" referrerpolicy="no-referrer"></iframe><br><img src="{}" referrerpolicy="no-referrer"><br>{}"#,
            video["param"]
                .as_str()
                .ok_or_else(|| anyhow!("缺少 avid param 字段"))?,
            video["bvid"]
                .as_str()
                .ok_or_else(|| anyhow!("缺少 bvid 字段"))?,
            video["cover"]
                .as_str()
                .ok_or_else(|| anyhow!("缺少 cover 字段"))?,
            video["title"]
                .as_str()
                .ok_or_else(|| anyhow!("缺少 title 字段"))?
        );
        let item = ItemBuilder::default()
            .title(Some(video["title"].to_string().trim_matches('"').to_string()))
            .link(Some(video["short_link"].to_string().trim_matches('"').to_string()))
            .description(description)
            .pub_date(now())
            .author(Some(video["right_desc_1"].to_string().trim_matches('"').to_string()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title("B站每周必看")
        .link("https://www.bilibili.com/v/popular/weekly")
        .description("B站每周必看视频精选")
        .items(item_vec) 
        .build();
    Ok(channel.to_string())
}
