use crate::crawler::fetch;
use anyhow::{Error, Ok, Result};
use chrono::Local;
use std::collections::HashMap;
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
