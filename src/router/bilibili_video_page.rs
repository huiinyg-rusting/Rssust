use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let bvid = para
        .get("bvid")
        .ok_or_else(|| anyhow!("缺少 bvid 参数"))?;

    let link = format!("https://www.bilibili.com/video/{}", bvid);
    let url = format!("https://api.bilibili.com/x/web-interface/view?bvid={}", bvid);
    let headers: Vec<(&str, &str)> = vec![("Referer", link.as_str())];

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &headers)?.as_str(),
    )?;

    let data = json
        .pointer("/data")
        .ok_or_else(|| anyhow!("找不到 data 字段"))?;
    let name = data["title"].as_str().unwrap_or_default();
    let pic = data["pic"].as_str().unwrap_or_default();
    let aid = data["aid"].as_i64().unwrap_or(0);
    let bvid_from_api = data["bvid"].as_str().unwrap_or(&bvid);

    let pages = data
        .pointer("/pages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data/pages 字段或不是数组"))?;

    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    // 按 page 降序排列取前 limit 个
    let mut sorted_pages: Vec<&Value> = pages.iter().collect();
    sorted_pages.sort_by(|a, b| {
        b["page"]
            .as_i64()
            .unwrap_or(0)
            .cmp(&a["page"].as_i64().unwrap_or(0))
    });

    let mut item_vec = Vec::new();

    for page in sorted_pages.iter().take(limit) {
        let part = page["part"].as_str().unwrap_or_default();
        let cid = page["cid"].as_i64().unwrap_or(0);
        let page_num = page["page"].as_i64().unwrap_or(0);

        let description = match para.get("disableembed") {
            Some(s) if s == "false" => "".to_owned(),
            _ => {
                format!(
                    r#"<iframe width="640" height="360" src="https://www.bilibili.com/blackboard/html5mobileplayer.html?aid={}&amp;cid={}&amp;bvid={}" frameborder="0" allowfullscreen="" referrerpolicy="no-referrer"></iframe><br><img src="{}" referrerpolicy="no-referrer"><br>{} - {}"#,
                    aid, cid, bvid_from_api, pic, part, name
                )
            }
        };

        let item = ItemBuilder::default()
            .title(Some(part.to_string()))
            .link(Some(format!("{}?p={}", link, page_num)))
            .description(description)
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("视频 {} 的选集列表", name))
        .link(link)
        .description(format!("视频 {} 的视频选集列表", name))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
