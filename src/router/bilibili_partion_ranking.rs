use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

fn format_date(days_ago: i64) -> String {
    let now = chrono::Local::now();
    let date = if days_ago > 0 {
        now - chrono::Duration::days(days_ago)
    } else {
        now
    };
    date.format("%Y%m%d").to_string()
}

fn tid_to_name(tid: &str) -> &str {
    match tid {
        "0" => "全部",
        "1" => "动画",
        "13" => "番剧",
        "167" => "国创",
        "3" => "音乐",
        "129" => "舞蹈",
        "4" => "游戏",
        "36" => "知识",
        "188" => "科技",
        "234" => "运动",
        "223" => "汽车",
        "160" => "生活",
        "211" => "美食",
        "217" => "动物圈",
        "119" => "鬼畜",
        "155" => "时尚",
        "5" => "娱乐",
        "181" => "影视",
        "177" => "纪录片",
        "23" => "电影",
        "11" => "电视剧",
        _ => "未知",
    }
}

fn build_item(title: &str, bvid: &str, aid: i64, pic: &str, desc: &str, tag: &str, pubdate: i64, author: &str, embed: bool) -> Item {
    let description = if embed {
        format!(
            r#"<iframe width="640" height="360" src="https://www.bilibili.com/blackboard/html5mobileplayer.html?aid={}&amp;cid=undefined&amp;bvid={}" frameborder="0" allowfullscreen="" referrerpolicy="no-referrer"></iframe><br><img src="{}" referrerpolicy="no-referrer"><br>{}<br>{}"#,
            aid, bvid, pic, desc, tag
        )
    } else {
        String::new()
    };

    ItemBuilder::default()
        .title(Some(title.to_string()))
        .link(Some(format!("https://www.bilibili.com/video/{}", bvid)))
        .description(description)
        .pub_date(timestamp_to_rss(pubdate))
        .author(Some(author.to_string()))
        .build()
}

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let tid = para
        .get("tid")
        .ok_or_else(|| anyhow!("缺少 tid 参数"))?;
    let days = para
        .get("days")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(7);
    let embed = !matches!(para.get("disableembed"), Some(s) if s == "false");

    let name = tid_to_name(tid);
    let headers: Vec<(&str, &str)> = vec![("Referer", "https://www.bilibili.com/")];

    let time_from = format_date(days);
    let time_to = format_date(0);
    let hot_url = format!(
        "https://s.search.bilibili.com/cate/search?main_ver=v3&search_type=video&view_type=hot_rank&cate_id={}&time_from={}&time_to={}&_={}",
        tid,
        time_from,
        time_to,
        chrono::Local::now().timestamp_millis()
    );

    let hot_json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&hot_url, &headers)?.as_str(),
    )?;

    let hot_result = hot_json
        .pointer("/result")
        .and_then(|v| v.as_array())
        .map(|v| v.clone())
        .unwrap_or_default();

    let mut item_vec = Vec::new();

    if hot_result.is_empty() {
        let newlist_url = format!(
            "https://api.bilibili.com/x/web-interface/newlist?ps=15&rid={}&_={}",
            tid,
            chrono::Local::now().timestamp_millis()
        );
        let newlist_json: Value = serde_json::from_str(
            fetch_reqwest_get_with_headers(&newlist_url, &headers)?.as_str(),
        )?;
        let archives = newlist_json
            .pointer("/data/archives")
            .and_then(|v| v.as_array())
            .map(|v| v.clone())
            .unwrap_or_default();

        for video in &archives {
            let aid = video["aid"].as_i64().unwrap_or(0);
            let bvid = video["bvid"].as_str().unwrap_or("");
            let pic = video["pic"].as_str().unwrap_or("");
            let desc = video["desc"].as_str().unwrap_or("");
            let pubdate = video["pubdate"].as_i64().unwrap_or(0);
            let author = video["owner"]["name"].as_str().unwrap_or("未知");
            let title = video["title"].as_str().unwrap_or("");
            let item = build_item(title, bvid, aid, pic, desc, "", pubdate, author, embed);
            item_vec.push(item);
        }
    } else {
        for item in hot_result.iter() {
            let id = item["id"].as_i64().unwrap_or(0);
            let bvid = item["bvid"].as_str().unwrap_or("");
            let pic = item["pic"].as_str().unwrap_or("");
            let description_text = item["description"].as_str().unwrap_or("");
            let tag = item["tag"].as_str().unwrap_or("");
            let pubdate = item["pubdate"].as_i64().unwrap_or(0);
            let author = item["author"].as_str().unwrap_or("未知");
            let title = item["title"].as_str().unwrap_or("");
            let item = build_item(title, bvid, id, pic, description_text, tag, pubdate, author, embed);
            item_vec.push(item);
        }
    }

    let channel = ChannelBuilder::default()
        .title(format!("bilibili {} 最热视频", name))
        .link("https://www.bilibili.com")
        .description(format!("bilibili {}分区 最热视频", name))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
