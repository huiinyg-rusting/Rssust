use crate::crawler::fetch;
use anyhow::{Error, Result, anyhow};
use chrono::DateTime;
use chrono::{Datelike, Local, NaiveDate};
use serde_json::Value;
use std::fs;
use std::result::Result::Ok;
use std::{collections::HashMap, env};
use tokio::runtime::Runtime;

///这个函数序列化从key1=1&key2=2 到{"key1": "2", "key2": "2"}的Hashmap;
pub fn params_to_hashmap(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            params.insert(key.to_string(), value.to_string());
        } else if !pair.is_empty() {
            // 处理没有等号的参数（如 "flag"），值设为空字符串
            params.insert(pair.to_string(), String::new());
        }
    }
    params
}

pub fn hashmap_to_params(hashmap: HashMap<String, String>) -> String {
    let mut response: String = "".to_owned();
    for (key, value) in hashmap.iter() {
        response.push_str(format!("{}={}", key, value).as_str());
    }
    response
}

/// 从指定 URL 抓取 HTML。
/// 推荐
pub fn fetch_obscura(url: &str) -> Result<String, Error> {
    let html = fetch(url)?;
    Ok(html)
}

//下面是reqwest get的内容
//不会使用线程池
pub fn fetch_reqwest_get(url: &str) -> Result<String, Error> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async { Ok(reqwest::get(url).await?.text().await?) })
}

///This can be an array of tuples, or a HashMap, or a custom type that implements Serialize.
///这可以是一个元组数组，或者是一个 HashMap ，或者是一个实现了 Serialize 的自定义类型。
///The feature form is required.
///必须使用 form 功能
pub fn fetch_reqwest_post(url: &str, body: String) -> Result<String, Error> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        Ok(reqwest::Client::new()
            .post(url)
            .body(body)
            .send()
            .await?
            .text()
            .await?)
    })
}

pub fn fetch_reqwest_get_with_headers(
    url: &str,
    headers: &[(&str, &str)],
) -> Result<String, Error> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let client = reqwest::Client::new();
        let mut req = client.get(url);
        for &(key, value) in headers {
            req = req.header(key, value);
        }
        Ok(req.send().await?.text().await?)
    })
}

///简单的1,true,True转true
pub fn parse_bool(value: Option<&String>, default: bool) -> bool {
    match value.map(String::as_str) {
        Some("1") | Some("true") | Some("True") => true,
        Some("0") | Some("false") | Some("False") => false,
        Some(other) => other.parse().unwrap_or(default),
        None => default,
    }
}

pub fn load_cookie_header(domain_filter: Option<&str>) -> Result<Option<String>> {
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| anyhow!("Could not get executable directory"))?;
    let cookie_path = exe_dir.join("cookies.json");
    let content = fs::read_to_string(cookie_path).map_err(|_| anyhow!("无法读取 cookies.json"))?;
    let cookies: Value = serde_json::from_str(content.as_str())?;
    let cookie_array = cookies
        .as_array()
        .ok_or_else(|| anyhow!("cookies.json 格式错误, 预期数组"))?;
    let cookie_pairs: Vec<String> = cookie_array
        .iter()
        .filter_map(|cookie| {
            if let Some(filter) = domain_filter {
                let domain = cookie.get("domain")?.as_str()?;
                if !domain.contains(filter) {
                    return None;
                }
            }
            let name = cookie.get("name")?.as_str()?;
            let value = cookie.get("value")?.as_str()?;
            Some(format!("{}={}", name, value))
        })
        .collect();
    if cookie_pairs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(cookie_pairs.join("; ")))
    }
}

pub fn now() -> String {
    Local::now().format("%a, %d %b %Y %H:%M:%S %z").to_string()
}

///x月y日到rss用的时间
//这个函数已经测试过有效了
///时间如果无效返回None
pub fn chinese_date_to_parse(input: &str) -> Option<String> {
    let re = regex::Regex::new(r"(\d{1,2})月(\d{1,2})日").ok()?;
    let caps = re.captures(input)?;
    let month = caps.get(1)?.as_str().parse::<u32>().ok()?;
    let day = caps.get(2)?.as_str().parse::<u32>().ok()?;
    let year = Local::now().year() as i32;
    Some(
        NaiveDate::from_ymd_opt(year, month, day)?
            .format("%a, %d %b %Y 00:00:00 +0800")
            .to_string(),
    )
}
///去除首尾双引号
//注意'"'是一对单引号包双引号
pub fn no_double_quotes(s: String) -> String {
    s.trim_matches('"').to_string()
}

//查找环境变量
pub fn env_search(s: &str) -> Option<String> {
    match env::var(s) {
        Ok(i) => Some(i),
        Err(_) => None,
    }
}

//Unix时间戳改RSS标准时间
pub fn timestamp_to_rss(ts: i64) -> String {
    DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
        .unwrap_or_else(now)
}
