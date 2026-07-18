use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::NaiveDate;
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

const ROOT_URL: &str = "https://www.cde.org.cn";

fn build_client() -> Result<reqwest::blocking::Client> {
    let client = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .build()?;
    client
        .get(ROOT_URL)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .send()?;
    Ok(client)
}

fn parse_cde_date(s: &str) -> Option<String> {
    let s = s.trim();
    if s.len() != 8 {
        return None;
    }
    let year = s[0..4].parse::<i32>().ok()?;
    let month = s[4..6].parse::<u32>().ok()?;
    let day = s[6..8].parse::<u32>().ok()?;
    NaiveDate::from_ymd_opt(year, month, day)
        .map(|d| d.format("%a, %d %b %Y 00:00:00 +0800").to_string())
}

fn get_description(doc: &Html) -> String {
    let box_sel = Selector::parse("div.news_detail_box").unwrap();

    let box_el = match doc.select(&box_sel).next() {
        Some(el) => el,
        None => return String::new(),
    };

    let mut result = String::new();
    for child in box_el.children() {
        if let Some(el_ref) = scraper::ElementRef::wrap(child) {
            let tag = el_ref.value().name();
            let is_title = tag == "div"
                && el_ref
                    .value()
                    .has_class("news_detail_title", scraper::CaseSensitivity::CaseSensitive);
            let is_date = tag == "div"
                && el_ref
                    .value()
                    .has_class("news_detail_date", scraper::CaseSensitivity::CaseSensitive);
            let is_img = tag == "img";
            if is_title || is_date || is_img {
                continue;
            }
            result.push_str(&el_ref.inner_html());
        } else if let Some(text) = child.value().as_text() {
            result.push_str(&text.text);
        }
    }
    result
}

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let channel = para
        .get("channel")
        .ok_or_else(|| anyhow!("缺少 channel 参数"))?;
    let category = para
        .get("category")
        .ok_or_else(|| anyhow!("缺少 category 参数"))?;

    let channel = channel.as_str();
    let category = category.as_str();

    let (cate_url, cate_title, channel_link) = match (channel, category) {
        ("news", "zwxw") => (
            "getList",
            "政务新闻",
            "https://www.cde.org.cn/main/news/listpage/545cf855a50574699b46b26bcb165f32",
        ),
        ("news", "ywdd") => (
            "getHotNewsList",
            "要闻导读",
            "https://www.cde.org.cn/main/news/listpage/545cf855a50574699b46b26bcb165f32",
        ),
        ("news", "tpxw") => (
            "getPictureNewsList",
            "图片新闻",
            "https://www.cde.org.cn/main/news/listpage/545cf855a50574699b46b26bcb165f32",
        ),
        ("news", "gzdt") => (
            "getWorkList",
            "工作动态",
            "https://www.cde.org.cn/main/news/listpage/545cf855a50574699b46b26bcb165f32",
        ),
        ("policy", "flfg") => (
            "getPolicyList",
            "法律法规",
            "https://www.cde.org.cn/main/policy/listpage/9f9c74c73e0f8f56a8bfbc646055026d",
        ),
        ("policy", "zxgz") => (
            "getRuleList",
            "政策规章",
            "https://www.cde.org.cn/main/policy/listpage/9f9c74c73e0f8f56a8bfbc646055026d",
        ),
        _ => return Err(anyhow!("无效的 channel/category 组合，请使用 ?channel=news|policy&category=zwxw|ywdd|tpxw|gzdt|flfg|zxgz")),
    };

    let page_size = para
        .get("limit")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(25);

    let request_data: Vec<(&str, String)> = match (channel, category) {
        ("news", "zwxw") => vec![
            ("pageSize", page_size.to_string()),
            ("pageNum", "1".to_string()),
            (
                "classId",
                "545cf855a50574699b46b26bcb165f32".to_string(),
            ),
        ],
        ("news", "ywdd") => vec![
            ("pageSize", page_size.to_string()),
            ("pageNum", "1".to_string()),
            ("ishot", "1".to_string()),
        ],
        ("news", "tpxw") => vec![
            ("pageSize", page_size.to_string()),
            ("pageNum", "1".to_string()),
        ],
        ("news", "gzdt") => vec![
            ("pageSize", page_size.to_string()),
            ("pageNum", "1".to_string()),
            (
                "classId",
                "8dc6aac86eb083759b1e01615617a347".to_string(),
            ),
        ],
        ("policy", "flfg") | ("policy", "zxgz") => vec![
            ("pageSize", page_size.to_string()),
            ("pageNum", "1".to_string()),
            ("fclass", "0".to_string()),
            ("keyName", "TITLE".to_string()),
            ("logicC", "bh".to_string()),
        ],
        _ => unreachable!(),
    };

    let client = build_client()?;

    let api_url = format!("{}/main/{}/{}", ROOT_URL, channel, cate_url);
    let body = request_data
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    let response = client
        .post(&api_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()?
        .text()?;

    let json: Value = serde_json::from_str(&response)?;
    let records = json
        .pointer("/data/records")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("找不到 data.records 字段或不是数组"))?;

    let mut item_vec = Vec::new();

    for record in records {
        let title = record
            .pointer("/title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let link = if channel == "news" {
            let is_pic = record
                .pointer("/isPic")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let news_id = record
                .pointer("/newsIdCode")
                .and_then(Value::as_str)
                .unwrap_or("");
            if is_pic {
                format!("{}/main/newspic/view/{}", ROOT_URL, news_id)
            } else {
                format!("{}/main/news/viewInfoCommon/{}", ROOT_URL, news_id)
            }
        } else {
            let regulat_id = record
                .pointer("/regulatIdCODE")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty());
            let policy_id = record
                .pointer("/policyIdCODE")
                .and_then(Value::as_str)
                .unwrap_or("");
            if let Some(rid) = regulat_id {
                format!("{}/main/policy/regulatview/{}", ROOT_URL, rid)
            } else {
                format!("{}/main/policy/view/{}", ROOT_URL, policy_id)
            }
        };

        let (pub_date, description) = match client
            .get(&link)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            )
            .send()
        {
            Ok(resp) => {
                let html = resp.text().unwrap_or_default();
                let doc = Html::parse_document(&html);
                let pub_date = doc
                    .select(&Selector::parse("div.news_detail_date").unwrap())
                    .next()
                    .and_then(|el| el.text().next())
                    .and_then(parse_cde_date)
                    .unwrap_or_else(now);
                let description = get_description(&doc);
                (pub_date, description)
            }
            Err(_) => (now(), String::new()),
        };

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(Some(link))
            .pub_date(Some(pub_date))
            .description(Some(description))
            .build();

        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("国家药品监督管理局药品审评中心 - {}", cate_title))
        .link(channel_link)
        .items(item_vec)
        .build();

    Ok(channel.to_string())
}
