use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

fn team_id(name: &str) -> Option<&'static str> {
    match name {
        "pistons" => Some("1901000000501290"),
        "knicks" => Some("1901000000501273"),
        "raptors" => Some("1901000000501298"),
        "heat" => Some("1901000000501267"),
        "celtics" => Some("1901000000501263"),
        "magic" => Some("1901000000501279"),
        "76ers" => Some("1901000000501281"),
        "cavaliers" => Some("1901000000501288"),
        "hawks" => Some("1901000000501284"),
        "bucks" => Some("1901000000501294"),
        "bulls" => Some("1901000000501286"),
        "hornets" => Some("1901000000501334"),
        "nets" => Some("1901000000501336"),
        "pacers" => Some("1901000000501292"),
        "wizards" => Some("1901000000501283"),
        "thunder" => Some("1901000000501329"),
        "lakers" => Some("1901000000501323"),
        "rockets" => Some("1901000000501311"),
        "spurs" => Some("1901000000501317"),
        "nuggets" => Some("1901000000501301"),
        "timberwolves" => Some("1901000000501315"),
        "suns" => Some("1901000000501327"),
        "warriors" => Some("1901000000501331"),
        "grizzlies" => Some("1901000000501313"),
        "trail blazers" => Some("1901000000501325"),
        "jazz" => Some("1901000000501320"),
        "mavericks" => Some("1901000000501300"),
        "clippers" => Some("1901000000501333"),
        "kings" => Some("1901000000501321"),
        "pelicans" => Some("1901000000501296"),
        _ => None,
    }
}

fn team_name_cn(name: &str) -> &'static str {
    match name {
        "pistons" => "活塞",
        "knicks" => "尼克斯",
        "raptors" => "猛龙",
        "heat" => "热火",
        "celtics" => "凯尔特人",
        "magic" => "魔术",
        "76ers" => "76人",
        "cavaliers" => "骑士",
        "hawks" => "老鹰",
        "bucks" => "雄鹿",
        "bulls" => "公牛",
        "hornets" => "黄蜂",
        "nets" => "篮网",
        "pacers" => "步行者",
        "wizards" => "奇才",
        "thunder" => "雷霆",
        "lakers" => "湖人",
        "rockets" => "火箭",
        "spurs" => "马刺",
        "nuggets" => "掘金",
        "timberwolves" => "森林狼",
        "suns" => "太阳",
        "warriors" => "勇士",
        "grizzlies" => "灰熊",
        "trail blazers" => "开拓者",
        "jazz" => "爵士",
        "mavericks" => "独行侠",
        "clippers" => "快船",
        "kings" => "国王",
        "pelicans" => "鹈鹕",
        _ => "未知",
    }
}

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let team = para
        .get("team")
        .ok_or_else(|| anyhow!("缺少 team 参数 (如 spurs, lakers)"))?;
    let team_lower = team.to_lowercase();

    let tid = team_id(&team_lower)
        .ok_or_else(|| anyhow!("无效的队名: {}，请使用英文队名如 spurs, lakers 等", team))?;

    let url = format!(
        "https://games.mobileapi.hupu.com/3/7.5.60/basketballapi/news/v2/teamNewsById?cateGoryCode=basketball&clientId=93977196&newsId=0&teamId={}",
        tid
    );

    let json: Value = serde_json::from_str(fetch_reqwest_get(&url)?.as_str())?;

    let list = json
        .pointer("/result")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 result 字段或不是数组"))?;

    let cn_name = team_name_cn(&team_lower);

    let mut item_vec = Vec::new();
    for item in list {
        let title = item["title"].as_str().unwrap_or("");
        let tid_str = item["tid"].as_str().unwrap_or("");
        let publish_time = item["publishTime"].as_str().unwrap_or("");

        let pub_date = if !publish_time.is_empty() {
            datetime_str_to_rss(publish_time).unwrap_or_else(now)
        } else {
            now()
        };

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(format!("https://m.hupu.com/bbs/{}", tid_str))
            .pub_date(pub_date)
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("虎扑 - {} 新闻", cn_name))
        .link("https://m.hupu.com".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
