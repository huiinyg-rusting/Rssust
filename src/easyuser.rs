use crate::crawler::fetch;
use anyhow::{Error, Result};
use chrono::{Datelike, Local, NaiveDate};
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
pub fn no_double_quotes(s: String) -> String {
    s.trim_matches('"').to_string() //注意'"'是一对单引号包双引号
}

//查找环境变量
//todo:这个函数未经过测试
pub fn env_search(s: &str) -> Option<String> {
    match env::var(s) {
        Ok(i) => Some(i),
        Err(_) => None,
    }
}
