use crate::crawler::{
    convert_relative_urls_to_absolute, fetch_obscura, fetch_reqwest_get, fetch_reqwest_post,
};
use anyhow::{Error, Ok, Result, anyhow};
use std::collections::HashMap;

///这个函数相当于模块的注册表
/// 给调用者的是html格式
pub fn request_rules(url: &str, op: Option<HashMap<String, String>>) -> Result<String, Error> {
    if url == "/" {
        eprintln!("{:?}", op);
        crate::connect::show_index_doc()
    } else if url == "/what" {
        Ok("What is this?".to_string())
    } else if url == "/bilibili" {
        convert_relative_urls_to_absolute(
            fetch_obscura("https://bilibili.com").unwrap().as_str(),
            "https://bilibili.com",
        )
    } else if url == "/git" {
        fetch_reqwest_get("https://api.kuleu.com/api/getGreetingMessage?type=json")
    } else if url == "/git1" {
        fetch_reqwest_post("https://httpbin.org/post", "".to_string())
    } else {
        Err(anyhow!("404NotFound"))
    }
}
