use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

///B站直播分区房间列表。
///Params: area_id(分区ID, 可通过 room/v1/Area/getList 查询; 支持父分区或子分区 id),
///order(排序方式 live_time/online),
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

    /* 分区目录接口：父分区 id 为数字，子分区 id 为字符串（如 "86" 英雄联盟） */
    let area_json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(
            "https://api.live.bilibili.com/room/v1/Area/getList",
            &headers,
        )
        .await?
        .as_str(),
    )?;

    let data = area_json
        .pointer("/data")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("获取直播分区列表失败"))?;

    let mut parent_title = String::new();
    let mut area_title = String::new();
    let mut area_link = String::new();
    let mut is_parent = false;
    /* 需要查询房间的分区：命中子分区时为单个，命中父分区时为其全部子分区 */
    let mut child_ids: Vec<String> = Vec::new();

    for parent in data {
        let pid = parent["id"].as_i64().unwrap_or(0);
        let pname = parent["name"].as_str().unwrap_or("");
        let list = match parent["list"].as_array() {
            Some(l) => l,
            None => continue,
        };

        /* 1) area_id 命中某个子分区 → 直连该子分区 */
        let mut hit_child = false;
        for area in list {
            let aid = area["id"].as_str().unwrap_or("");
            if aid == area_id
                || area["id"].as_i64().map(|i| i.to_string()) == Some(area_id.clone())
            {
                parent_title = pname.to_string();
                area_title = area["name"].as_str().unwrap_or("").to_string();
                area_link = format!(
                    "https://live.bilibili.com/p/eden/area-tags?parentAreaId={}&areaId={}",
                    pid, area_id
                );
                child_ids.push(area_id.clone());
                hit_child = true;
                break;
            }
        }
        if hit_child {
            break;
        }

        /* 2) area_id 命中父分区 → 展开其全部子分区 */
        if pid.to_string() == area_id {
            parent_title = pname.to_string();
            area_title = pname.to_string();
            area_link = format!(
                "https://live.bilibili.com/p/eden/area-tags?parentAreaId={}&areaId={}",
                pid, area_id
            );
            is_parent = true;
            for area in list {
                let aid = area["id"].as_str().unwrap_or("");
                if !aid.is_empty() {
                    child_ids.push(aid.to_string());
                }
            }
            break;
        }
    }

    if child_ids.is_empty() {
        return Err(anyhow!(
            "找不到 area_id={} 对应的分区（可请求 room/v1/Area/getList 查询有效分区 id）",
            area_id
        ));
    }
    if is_parent && child_ids.len() > 20 {
        return Err(anyhow!(
            "父分区“{}”子分区过多（{} 个），请改用具体子分区 id",
            area_title,
            child_ids.len()
        ));
    }

    /* 查询房间列表：单子分区直连；父分区展开各子分区并发查询后合并。
       房间列表接口（room/v1/area/getRoomList）只接受子分区 id，父分区 id 会返回 404 */
    let mut all_rooms: Vec<Value> = Vec::new();
    if child_ids.len() == 1 {
        let room_url = format!(
            "https://api.live.bilibili.com/room/v1/area/getRoomList?area_id={}&sort_type={}&page_size={}&page_no={}",
            child_ids[0], order, page_size, page_no
        );
        let resp = fetch_reqwest_get_with_headers(&room_url, &headers).await?;
        let json: Value = serde_json::from_str(&resp)?;
        if json["code"].as_i64() != Some(0) {
            return Err(anyhow!(
                "B站接口返回: {}",
                json["message"].as_str().unwrap_or("未知错误")
            ));
        }
        all_rooms = json
            .pointer("/data")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
    } else {
        /* 父分区展开：顺序查询各子分区并合并；单个子分区失败不影响整体 */
        for cid in &child_ids {
            let room_url = format!(
                "https://api.live.bilibili.com/room/v1/area/getRoomList?area_id={}&sort_type={}&page_size={}&page_no={}",
                cid, order, page_size, page_no
            );
            if let Ok(resp) = fetch_reqwest_get_with_headers(&room_url, &headers).await
                && let Ok(json) = serde_json::from_str::<Value>(&resp)
                && let Some(rooms) = json.pointer("/data").and_then(Value::as_array)
            {
                all_rooms.extend(rooms.clone());
            }
        }
    }

    let display_title = if is_parent || parent_title.is_empty() {
        format!("{}分区-{}", area_title, order_title)
    } else {
        format!("{}·{}分区-{}", parent_title, area_title, order_title)
    };

    let mut item_vec = Vec::new();
    for room in &all_rooms {
        let roomid = room["roomid"].as_i64().unwrap_or(0);
        let uname = room["uname"].as_str().unwrap_or("");
        let title = room["title"].as_str().unwrap_or("");
        let online = room["online"].as_i64().unwrap_or(0);
        let uid = room["uid"].as_i64().unwrap_or(0);
        /* 展开父分区时，每个房间标注自己所属的子分区 */
        let room_area = room["area_v2_name"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or(area_title.as_str());

        let description = format!(
            "主播: {} (UID {})<br>在线人数: {}<br>分区: {}",
            uname, uid, online, room_area
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