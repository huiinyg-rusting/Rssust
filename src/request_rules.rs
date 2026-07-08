use anyhow::{Error, Ok, Result, anyhow};
use std::collections::HashMap;

use crate::easyuser::*; //todo：这一行以后应该是要删除的，测试用

pub enum ShowToUser {
    Html{res: Result<String, Error>},
    Rss{res:Result<String, Error>}
}

///这个函数相当于模块的注册表
/// 给调用者的是html格式
pub fn request_rules(url: &str, parameters: HashMap<String, String>) -> Result<String, Error> {
    if url == "/" {
        eprintln!("{:?}", parameters); //todo:这里也应该删除
        crate::connect::show_index_doc()
    } else if url == "/what" {
        Ok("What is this?".to_string())
    } else if url == "/bilibili" {
        fetch_obscura("https://bilibili.com")
    } else if url == "/git" {
        fetch_reqwest_get("https://api.kuleu.com/api/getGreetingMessage?type=json")
    } else if url == "/git1" {
        fetch_reqwest_post("https://httpbin.org/post", "".to_string())
    } else if url == "/bilibili_week" {
        crate::router::bilibili_weekly::get()
    } else {
        Err(anyhow!("404NotFound"))
    }
}
pub fn root_rules(first_part: &str,second_part: HashMap<String, String>) -> ShowToUser{
    if first_part == "/" {
        ShowToUser::Html {res: crate::connect::show_index_doc()}
    } else if first_part.starts_with("/doc/") {
        ShowToUser::Html {res: crate::connect::show_index_doc()}
    } else {
        ShowToUser::Rss{res:request_rules(first_part, second_part)}
    }
}
