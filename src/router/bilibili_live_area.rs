use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

///B站直播分区房间列表。
///Params: area_id(分区ID, 可通过 room/v1/Area/getList 查询), order(排序方式 live_time/online),
///page_size(每页条数, 默认30), page_no(页码, 默认1)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let area_id = para
        .get("area_id")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("缺少 area_id 参数（直播分区ID）"))?;
    let order = para
        .get("order")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("缺少 order 参数（live_time 或 online）"))?;
    let page_size = para
        .get("page_size")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(30)
        .min(30);
    let page_no = para
        .get("page_no")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);

    let order_title = match order.as_str() {
        "live_time" => "最新开播",
        "online" => "人气直播",
        _ => return Err(anyhow!("order 参数仅支持 live_time 或 online")),
    };

    let referer = "https://live.bilibili.com/p/eden/area-tags";
    let headers: Vec<(&str, &str)> = vec![("Referer", referer)];

    let area_json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(
            "https://api.live.bilibili.com/room/v1/Area/getList",
            &headers,
        )
        .await?
        .as_str(),
    )?;

    let mut parent_title = String::new();
    let mut area_title = String::new();
    let mut area_link = String::new();
    if let Some(parents) = area_json.pointer("/data").and_then(Value::as_array) {
        for parent in parents {
            let pid = parent["id"].as_i64().unwrap_or(0);
            let pname = parent["name"].as_str().unwrap_or("");
            if let Some(list) = parent["list"].as_array() {
                for area in list {
                    if area["id"].as_i64().map(|id| id.to_string()) == Some(area_id.clone()) {
                        parent_title = pname.to_string();
                        area_title = area["name"].as_str().unwrap_or("").to_string();
                        area_link = format!(
                            "https://live.bilibili.com/p/eden/area-tags?parentAreaId={}&areaId={}",
                            pid, area_id
                        );
                        break;
                    }
                }
            }
            if !parent_title.is_empty() {
                break;
            }
        }
    }

    if area_title.is_empty() {
        area_title = area_id.clone();
    }
    let display_title = if parent_title.is_empty() {
        format!("{}分区-{}", area_title, order_title)
    } else {
        format!("{}·{}分区-{}", parent_title, area_title, order_title)
    };

    let room_url = format!(
        "https://api.live.bilibili.com/room/v1/area/getRoomList?area_id={}&sort_type={}&page_size={}&page_no={}",
        area_id, order, page_size, page_no
    );
    let resp = fetch_reqwest_get_with_headers(&room_url, &headers).await?;
    let json: Value = serde_json::from_str(&resp)?;

    let list = json
        .pointer("/data")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("找不到 data 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    for room in list {
        let roomid = room["roomid"].as_i64().unwrap_or(0);
        let uname = room["uname"].as_str().unwrap_or("");
        let title = room["title"].as_str().unwrap_or("");
        let online = room["online"].as_i64().unwrap_or(0);
        let uid = room["uid"].as_i64().unwrap_or(0);

        let description = format!(
            "主播: {} (UID {})<br>在线人数: {}<br>分区: {}",
            uname, uid, online, area_title
        );

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(format!("{} {}", uname, title))))
            .link(Some(format!("https://live.bilibili.com/{}", roomid)))
            .description(description)
            .pub_date(now())
            .author(Some(uname.to_string()))
            .guid(rss::Guid {
                value: format!("{}-{}", roomid, title),
                permalink: false,
            })
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("哔哩哔哩直播-{}", display_title))
        .link(if area_link.is_empty() {
            "https://live.bilibili.com/".to_string()
        } else {
            area_link
        })
        .description(format!("哔哩哔哩直播-{}", display_title))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}