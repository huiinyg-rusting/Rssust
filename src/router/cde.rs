use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::NaiveDate;
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;
use std::result::Result::Ok;
use tokio::runtime::Runtime;

const ROOT_URL: &str = "https://www.cde.org.cn";
const TITLE: &str = "国家药品监督管理局药品审评中心";

fn get_cookie() -> Result<String, Error> {
    let rt = Runtime::new()?;
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|e| anyhow!("构建 reqwest 客户端失败: {}", e))?;
        let resp = client
            .get(ROOT_URL)
            .send()
            .await
            .map_err(|e| anyhow!("获取 CDE 首页失败: {}", e))?;
        // 获取所有 set-cookie 头（可能有多个）
        let all_cookies: Vec<String> = resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok().map(|s| s.to_string()))
            .collect();
        let combined = all_cookies.join(",");
        // 提取所有 FSSBBIl1UgzbN7N80 开头的 cookie (匹配 RSSHub 行为)
        let re = regex::Regex::new(r"FSSBBIl1UgzbN7N80.*?;").unwrap();
        let cookie_str = re
            .find_iter(&combined)
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join("");
        if cookie_str.is_empty() {
            Err(anyhow!("未找到 FSSBBIl1UgzbN7N80 cookie"))
        } else {
            Ok(cookie_str)
        }
    })
}

fn fetch_post(url: &str, form_data: &[(&str, &str)], cookie: &str) -> Result<String, Error> {
    let rt = Runtime::new()?;
    rt.block_on(async {
        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .header("Cookie", cookie)
            .form(form_data)
            .send()
            .await
            .map_err(|e| anyhow!("POST 请求失败: {}", e))?;
        resp.text()
            .await
            .map_err(|e| anyhow!("读取响应失败: {}", e))
    })
}

fn fetch_get(url: &str, cookie: &str) -> Result<String, Error> {
    let rt = Runtime::new()?;
    rt.block_on(async {
        let client = reqwest::Client::new();
        let resp = client
            .get(url)
            .header("Cookie", cookie)
            .send()
            .await
            .map_err(|e| anyhow!("GET 请求失败: {}", e))?;
        resp.text()
            .await
            .map_err(|e| anyhow!("读取响应失败: {}", e))
    })
}

fn get_cate_url(channel: &str, cate: &str) -> Result<&'static str, Error> {
    match (channel, cate) {
        ("news", "zwxw") => Ok("getList"),
        ("news", "ywdd") => Ok("getHotNewsList"),
        ("news", "tpxw") => Ok("getPictureNewsList"),
        ("news", "gzdt") => Ok("getWorkList"),
        ("policy", "flfg") => Ok("getPolicyList"),
        ("policy", "zxgz") => Ok("getRuleList"),
        _ => Err(anyhow!("无效的频道或类别: {}/{}", channel, cate)),
    }
}

fn get_cate_title(channel: &str, cate: &str) -> Result<&'static str, Error> {
    match (channel, cate) {
        ("news", "zwxw") => Ok("政务新闻"),
        ("news", "ywdd") => Ok("要闻导读"),
        ("news", "tpxw") => Ok("图片新闻"),
        ("news", "gzdt") => Ok("工作动态"),
        ("policy", "flfg") => Ok("法律法规"),
        ("policy", "zxgz") => Ok("政策规章"),
        _ => Err(anyhow!("无效的频道或类别: {}/{}", channel, cate)),
    }
}

fn get_channel_link(channel: &str) -> Result<&'static str, Error> {
    match channel {
        "news" => Ok("https://www.cde.org.cn/main/news/listpage/545cf855a50574699b46b26bcb165f32"),
        "policy" => {
            Ok("https://www.cde.org.cn/main/policy/listpage/9f9c74c73e0f8f56a8bfbc646055026d")
        }
        _ => Err(anyhow!("无效的频道: {}", channel)),
    }
}

fn build_form_data(
    channel: &str,
    cate: &str,
    page_size: &str,
) -> Result<Vec<(&'static str, String)>, Error> {
    match (channel, cate) {
        ("news", "zwxw") => Ok(vec![
            ("pageSize", page_size.to_string()),
            ("pageNum", "1".to_string()),
            ("classId", "545cf855a50574699b46b26bcb165f32".to_string()),
        ]),
        ("news", "ywdd") => Ok(vec![
            ("pageSize", page_size.to_string()),
            ("pageNum", "1".to_string()),
            ("ishot", "1".to_string()),
        ]),
        ("news", "tpxw") => Ok(vec![
            ("pageSize", page_size.to_string()),
            ("pageNum", "1".to_string()),
        ]),
        ("news", "gzdt") => Ok(vec![
            ("pageSize", page_size.to_string()),
            ("pageNum", "1".to_string()),
            ("classId", "8dc6aac86eb083759b1e01615617a347".to_string()),
        ]),
        ("policy", "flfg") | ("policy", "zxgz") => Ok(vec![
            ("pageNum", "1".to_string()),
            ("pageSize", page_size.to_string()),
            ("fclass", "0".to_string()),
            ("keyName", "TITLE".to_string()),
            ("logicC", "bh".to_string()),
        ]),
        _ => Err(anyhow!("无效的频道或类别: {}/{}", channel, cate)),
    }
}

fn parse_detail_page(html: &str) -> Result<(String, String), Error> {
    let document = Html::parse_document(html);

    let date_selector =
        Selector::parse("div.news_detail_date").map_err(|_| anyhow!("无法解析日期选择器"))?;
    let pub_date = document
        .select(&date_selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let rss_date = if !pub_date.is_empty() {
        if let Ok(naive) = NaiveDate::parse_from_str(&pub_date, "%Y%m%d") {
            naive
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .format("%a, %d %b %Y %H:%M:%S %z")
                .to_string()
        } else {
            now()
        }
    } else {
        now()
    };

    let content_selector =
        Selector::parse("div.new_detail_content").map_err(|_| anyhow!("无法解析内容选择器"))?;
    let desc_html = document
        .select(&content_selector)
        .next()
        .map(|el| el.inner_html())
        .unwrap_or_default();

    Ok((rss_date, desc_html))
}

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let channel = para
        .get("channel")
        .ok_or_else(|| anyhow!("缺少必要参数: channel"))?;
    let cate = para
        .get("category")
        .ok_or_else(|| anyhow!("缺少必要参数: category"))?;
    let page_size = para.get("limit").map(|s| s.as_str()).unwrap_or("25");

    let cookie = get_cookie()?;

    let cate_url = get_cate_url(channel, cate)?;
    let api_url = format!("{}/main/{}/{}", ROOT_URL, channel, cate_url);
    let form_data_owned = build_form_data(channel, cate, page_size)?;
    let form_data_refs: Vec<(&str, &str)> = form_data_owned
        .iter()
        .map(|(k, v)| (*k, v.as_str()))
        .collect();

    let json_str = fetch_post(&api_url, &form_data_refs, &cookie)?;
    let json: Value =
        serde_json::from_str(&json_str).map_err(|e| anyhow!("JSON 解析失败: {}", e))?;

    let records = json
        .pointer("/data/records")
        .and_then(|r| r.as_array())
        .ok_or_else(|| anyhow!("响应中缺少 data/records 字段"))?;

    let mut item_vec = Vec::new();

    for record in records {
        let title = record["title"]
            .as_str()
            .ok_or_else(|| anyhow!("记录缺少 title 字段"))?;

        let link = if channel == "news" {
            let news_id = record["newsIdCode"]
                .as_str()
                .ok_or_else(|| anyhow!("新闻记录缺少 newsIdCode 字段"))?;
            let is_pic = record["isPic"].as_str().unwrap_or("0") == "1";
            if is_pic {
                format!("{}/main/newspic/view/{}", ROOT_URL, news_id)
            } else {
                format!("{}/main/news/viewInfoCommon/{}", ROOT_URL, news_id)
            }
        } else if let Some(regulat_id) = record["regulatIdCODE"].as_str() {
            format!("{}/main/policy/regulatview/{}", ROOT_URL, regulat_id)
        } else {
            let policy_id = record["policyIdCODE"]
                .as_str()
                .ok_or_else(|| anyhow!("政策记录缺少 policyIdCODE 字段"))?;
            format!("{}/main/policy/view/{}", ROOT_URL, policy_id)
        };

        let (pub_date, description) = match fetch_get(&link, &cookie) {
            Ok(html) => {
                let (date, desc) = parse_detail_page(&html).unwrap_or((now(), String::new()));
                (date, desc)
            }
            Err(_) => (now(), String::new()),
        };

        let item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(link)
            .pub_date(pub_date)
            .description(Some(description))
            .build();
        item_vec.push(item);
    }

    let channel_link = get_channel_link(channel)?;
    let cate_title = get_cate_title(channel, cate)?;

    let channel = ChannelBuilder::default()
        .title(format!("{} - {}", TITLE, cate_title))
        .link(channel_link.to_string())
        .items(item_vec)
        .build();

    Ok(channel.to_string())
}
