use crate::easyuser::*;
use anyhow::{Error, Ok, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let series_id = para
        .get("series_id")
        .ok_or_else(|| anyhow!("缺少 series_id 参数"))?;
    let series_json: Value = serde_json::from_str(
        fetch_reqwest_get(
            format!(
                "https://api.bilibili.com/x/series/series?series_id={}",
                series_id
            )
            .as_str(),
        )?
        .as_str(),
    )?;

    let meta = series_json
        .pointer("/data/meta")
        .ok_or_else(|| anyhow!("找不到 data.meta 字段"))?;
    let series_name = meta["name"].as_str().unwrap_or("B站系列");
    let series_desc = meta["description"].as_str().unwrap_or("");
    let mid = meta["mid"].as_u64().unwrap_or(0);
    let recent_aids = series_json
        .pointer("/data/recent_aids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data.recent_aids 字段或不是数组"))?;

    let mut item_vec = Vec::new();

    for aid_val in recent_aids {
        let aid = aid_val.as_u64().ok_or_else(|| anyhow!("aid 不是数字"))?;
        let view_json: Value = serde_json::from_str(
            fetch_reqwest_get(
                format!("https://api.bilibili.com/x/web-interface/view?aid={}", aid).as_str(),
            )?
            .as_str(),
        )?;
        let data = view_json
            .pointer("/data")
            .ok_or_else(|| anyhow!("接口返回没有 data 字段"))?;

        let bvid = data["bvid"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 bvid 字段"))?;
        let pic = data["pic"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 pic 字段"))?;
        let desc = data["desc"].as_str().unwrap_or_default();
        let pubdate = data["pubdate"]
            .as_i64()
            .ok_or_else(|| anyhow!("缺少 pubdate 字段"))?;
        let author = data["owner"]["name"].as_str().unwrap_or("未知");

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
            .title(Some(no_double_quotes(data["title"].to_string())))
            .link(Some(format!("https://www.bilibili.com/video/{}", bvid)))
            .description(description)
            .pub_date(timestamp_to_rss(pubdate))
            .author(Some(author.to_owned()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("B站{}", series_name))
        .link(format!(
            "https://space.bilibili.com/{}/lists/{}?type=series",
            mid, series_id
        ))
        .description(series_desc.to_owned())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
