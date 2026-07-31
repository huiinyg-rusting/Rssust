use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const PROVINCE_MAP: &[(&str, &str)] = &[
    ("北京", "北京市"),
    ("上海", "上海市"),
    ("天津", "天津市"),
    ("重庆", "重庆市"),
    ("黑龙江", "黑龙江省"),
    ("吉林", "吉林省"),
    ("辽宁", "辽宁省"),
    ("内蒙古", "内蒙古自治区"),
    ("河北", "河北省"),
    ("山西", "山西省"),
    ("陕西", "陕西省"),
    ("山东", "山东省"),
    ("新疆", "新疆维吾尔自治区"),
    ("西藏", "西藏自治区"),
    ("青海", "青海省"),
    ("甘肃", "甘肃省"),
    ("宁夏", "宁夏回族自治区"),
    ("河南", "河南省"),
    ("江苏", "江苏省"),
    ("湖北", "湖北省"),
    ("浙江", "浙江省"),
    ("安徽", "安徽省"),
    ("福建", "福建省"),
    ("江西", "江西省"),
    ("湖南", "湖南省"),
    ("贵州", "贵州省"),
    ("四川", "四川省"),
    ("广东", "广东省"),
    ("云南", "云南省"),
    ("广西", "广西壮族自治区"),
    ("海南", "海南省"),
];

fn normalize_province(p: &str) -> String {
    for (short, full) in PROVINCE_MAP {
        if p == *short || p == *full {
            return (*full).to_string();
        }
    }
    p.to_string()
}

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let type_ = para.get("type").map(|s| s.as_str()).unwrap_or("");
    let level = para.get("level").map(|s| s.as_str()).unwrap_or("");
    let mut province = para.get("province").map(|s| s.as_str()).unwrap_or("");
    if matches!(province, "全国" | "全部") {
        province = "";
    }
    let date = para.get("date").map(|s| s.as_str()).unwrap_or("");
    let limit: usize = para
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50)
        .clamp(1, 200);

    let province = normalize_province(province);

    let filters = format!(
        "signaltype={}&signallevel={}&province={}",
        urlencoding::encode(type_),
        urlencoding::encode(level),
        urlencoding::encode(&province),
    );

    let date_prefix = if date.is_empty() {
        None
    } else {
        Some(date.replace('-', "/").to_string())
    };

    let mut collected: Vec<Value> = Vec::new();
    let mut page = 1;
    let page_size = 200;
    let max_pages = 20;
    'outer: loop {
        let url = format!(
            "https://www.nmc.cn/rest/findAlarm?pageNo={}&pageSize={}&{}",
            page, page_size, filters
        );
        let resp = fetch_reqwest_get(&url)?;
        let json: Value = serde_json::from_str(&resp)?;
        if json["code"].as_i64() != Some(0) {
            return Err(anyhow!("API error: {}", json["msg"]));
        }
        let list = json["data"]["page"]["list"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let total_page = json["data"]["page"]["totalPage"].as_i64().unwrap_or(1);

        for item in &list {
            let t = item["issuetime"].as_str().unwrap_or("");
            if let Some(prefix) = &date_prefix {
                if t.starts_with(prefix.as_str()) {
                    collected.push(item.clone());
                } else if t < prefix.as_str() {
                    break 'outer;
                }
            } else {
                collected.push(item.clone());
            }
            if collected.len() >= limit {
                break 'outer;
            }
        }

        if page >= total_page || page >= max_pages || list.is_empty() {
            break;
        }
        page += 1;
    }
    collected.truncate(limit);

    let mut item_vec = Vec::new();
    for item in collected {
        let title = item["title"].as_str().unwrap_or("").to_string();
        let url = item["url"].as_str().unwrap_or("");
        let pic = item["pic"].as_str().unwrap_or("");
        let issuetime = item["issuetime"].as_str().unwrap_or("");
        let description = format!(
            "<img src=\"{}\" alt=\"预警等级\" /><p>{}</p><p>发布时间：{}</p>",
            pic, title, issuetime
        );
        let rss_item = ItemBuilder::default()
            .title(Some(title))
            .link(format!("https://www.nmc.cn{}", url))
            .description(Some(description))
            .pub_date(datetime_str_to_rss(&format!(
                "{}:00",
                issuetime.replace('/', "-")
            )))
            .build();
        item_vec.push(rss_item);
    }

    let mut title_parts = Vec::new();
    if !province.is_empty() {
        title_parts.push(format!("{}预警", province));
    }
    if !type_.is_empty() {
        title_parts.push(type_.to_string());
    }
    if !level.is_empty() {
        title_parts.push(format!("{}预警", level));
    }
    let suffix = if title_parts.is_empty() {
        String::new()
    } else {
        format!(" - {}", title_parts.join(" "))
    };

    let channel = ChannelBuilder::default()
        .title(format!("中央气象台预警信号{}", suffix))
        .link("https://www.nmc.cn/publish/alarm.html")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
