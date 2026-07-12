use crate::router::*;
use anyhow::*;
use std::collections::HashMap;

pub enum ShowToUser {
    Html { res: Result<String, Error> },
    Rss { res: Result<String, Error> },
}

///这个函数相当于模块的注册表
/// 给调用者的是html格式
pub fn request_rules(url: &str, parameters: HashMap<String, String>) -> Result<String, Error> {
    if url == "/bilibili_weekly" {
        bilibili_weekly::get(parameters)
    } else if url == "/bilibili_dynamic" {
        bilibili_dynamic::get(parameters)
    } else {
        Err(anyhow!("404NotFound"))
    }
}
pub fn root_rules(first_part: &str, second_part: HashMap<String, String>) -> ShowToUser {
    if first_part == "/" {
        ShowToUser::Html {
            res: crate::connect::show_index_doc(),
        }
    } else if first_part.starts_with("/docs/") {
        ShowToUser::Html {
            res: crate::connect::show_doc(first_part),
        }
    } else {
        match request_rules(first_part, second_part) {
            std::result::Result::Ok(i) => ShowToUser::Rss { res: Ok(i) },
            Err(i) => ShowToUser::Html { res: Err(i) },
        }
    }
}
