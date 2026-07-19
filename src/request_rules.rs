use crate::router::cde;
use crate::router::*;
use anyhow::*;
use std::collections::HashMap;

pub enum ShowToUser {
    Html {
        res: Result<String, Error>,
    },
    Rss {
        res: Result<String, Error>,
    },
    File {
        res: Result<String, Error>,
        content_type: String,
    },
}

///这个函数相当于模块的注册表
/// 给调用者的是html格式
pub fn request_rules(url: &str, parameters: HashMap<String, String>) -> Result<String, Error> {
    if url == "/bilibili_weekly" {
        bilibili_weekly::get(parameters)
    } else if url == "/bilibili_dynamic" {
        bilibili_dynamic::get(parameters)
    } else if url == "/bilibili_popular" {
        bilibili_popular::get(parameters)
    } else if url == "/bilibili_precious" {
        bilibili_precious::get(parameters)
    } else if url == "/bilibili_series" {
        bilibili_series::get(parameters)
    } else if url == "/bilibili_collection" {
        bilibili_collection::get(parameters)
    } else if url == "/bilibili_fav" {
        bilibili_fav::get(parameters)
    } else if url == "/bilibili_link_news" {
        bilibili_link_news::get(parameters)
    } else if url == "/bilibili_partion" {
        bilibili_partion::get(parameters)
    } else if url == "/bilibili_partion_ranking" {
        bilibili_partion_ranking::get(parameters)
    } else if url == "/bilibili_user_article" {
        bilibili_user_article::get(parameters)
    } else if url == "/bilibili_user_coin" {
        bilibili_user_coin::get(parameters)
    } else if url == "/bilibili_user_fav" {
        bilibili_user_fav::get(parameters)
    } else if url == "/bilibili_user_like" {
        bilibili_user_like::get(parameters)
    } else if url == "/bilibili_video_page" {
        bilibili_video_page::get(parameters)
    } else if url == "/bilibili_video_reply" {
        bilibili_video_reply::get(parameters)
    } else if url == "/bilibili_vsearch" {
        bilibili_vsearch::get(parameters)
    } else if url == "/zhihu_hot" {
        zhihu_hot::get(parameters)
    } else if url == "/cde" {
        cde::get(parameters)
    } else {
        Err(anyhow!("404NotFound"))
    }
}
pub fn root_rules(first_part: &str, second_part: HashMap<String, String>) -> ShowToUser {
    if first_part == "/" {
        ShowToUser::Html {
            res: crate::connect::show_index_doc(),
        }
    } else if first_part.starts_with("/docs/") || first_part.starts_with("/index/") {
        crate::connect::serve_static(first_part)
    } else {
        match request_rules(first_part, second_part) {
            std::result::Result::Ok(i) => ShowToUser::Rss { res: Ok(i) },
            Err(i) => ShowToUser::Html { res: Err(i) },
        }
    }
}
