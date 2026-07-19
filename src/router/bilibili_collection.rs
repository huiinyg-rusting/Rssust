use crate::easyuser::*;
use anyhow::{Error, Ok, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let uid = para.get("uid").ok_or_else(|| anyhow::anyhow!("没有uid"))?;
    let sid = para.get("sid").ok_or_else(|| anyhow::anyhow!("没有sid"))?;
    let sort_reverse = match para.get("sortreverse") {
        Some(val) => val
            .parse::<bool>()
            .map_err(|_| anyhow::anyhow!("sortreverse 参数格式错误，应为 true 或 false"))?,
        None => true,
    };
    let sort_reverse_str = if sort_reverse { "1" } else { "0" };
    let page_num = para
        .get("pagenum")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);

    let page_size = para
        .get("pagesize")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(16);
    let url = format!(
        "https://api.bilibili.com/x/polymer/web-space/seasons_archives_list?mid={}&season_id={}&sort_reverse={}&page_num={}&page_size={}",
        uid, sid, sort_reverse_str, page_num, page_size
    );
    let referer = format!("https://space.bilibili.com/{}/", uid);
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    let headers: Vec<(&str, &str)> = vec![("Referer", referer.as_str()), ("User-Agent", ua)];
    let json: Value =
        serde_json::from_str(fetch_reqwest_get_with_headers(url.as_str(), &headers)?.as_str())?;
    if json["code"] != 0 {
        return Err(anyhow!("请求失败，{}", json));
    }
    let author: String = serde_json::from_str::<Value>(
        fetch_reqwest_get_with_headers(
            &format!(
                "https://api.bilibili.com/x/web-interface/view?aid={}",
                json.pointer("/data/aids/0")
                    .ok_or_else(|| anyhow!("bilibili_collection.rs在解析author时爬取信息出错"))?
                    .to_string()
            ),
            &headers,
        )?
        .as_str(),
    )?
    .pointer("/data/owner/name")
    .ok_or_else(|| anyhow!("bilibili_collection.rs在解析author时解析出错"))?
    .as_str()
    .ok_or_else(|| anyhow!("owner/name 不是字符串"))?
    .to_owned();
    let mut item_vec = Vec::new();
    let list = json
        .pointer("/data/archives")
        .and_then(|list| list.as_array())
        .ok_or_else(|| anyhow!("找不到 data.archives 字段或不是数组"))?;
    for video in list {
        let description = match para.get("disableembed") {
            Some(s) if s == "false" => "".to_owned(),
            _ => {
                format!(
                    r#"<iframe width="640" height="360" src="https://www.bilibili.com/blackboard/html5mobileplayer.html?aid={}&amp;cid=undefined&amp;bvid={}" frameborder="0" allowfullscreen="" referrerpolicy="no-referrer"></iframe><br><img src="{}" referrerpolicy="no-referrer"><br>{}"#,
                    video["aid"]
                        .as_i64()
                        .ok_or_else(|| anyhow!("缺少 aid 字段"))?,
                    video["bvid"]
                        .as_str()
                        .ok_or_else(|| anyhow!("缺少 bvid 字段"))?,
                    video["pic"]
                        .as_str()
                        .ok_or_else(|| anyhow!("缺少 pic 字段"))?,
                    video["title"]
                        .as_str()
                        .ok_or_else(|| anyhow!("缺少 title 字段"))?
                )
            }
        };

        let pubdate = timestamp_to_rss(
            video["pubdate"]
                .as_i64()
                .ok_or_else(|| anyhow!("bilibili_collection无法找到pubdate项"))?,
        );
        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(video["title"].to_string())))
            .link(format!(
                "https://www.bilibili.com/video/{}",
                no_double_quotes(video["bvid"].to_string())
            ))
            .description(description)
            .pub_date(pubdate)
            .author(author.clone())
            .build();
        item_vec.push(item);
    }

    let ctitle: String = no_double_quotes(
        json.pointer("/data/meta/name")
            .ok_or_else(|| anyhow!("bilibili_collection.rs在解析channel标题时出错"))?
            .to_string(),
    );

    let channel = ChannelBuilder::default()
        .title(&ctitle)
        .link(format!(
            "https://www.bilibili.com/video/{}",
            no_double_quotes(
                json.pointer("/data/archives/0/bvid")
                    .ok_or_else(|| anyhow!("bilibili_collection.rs在解析author时出错"))?
                    .to_string()
            )
        ))
        .description(&ctitle)
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
