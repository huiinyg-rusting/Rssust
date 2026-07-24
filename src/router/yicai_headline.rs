use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let json: Value = serde_json::from_str(&fetch_reqwest_get(
        "https://www.yicai.com/api/ajax/getlistbycid?cid=48&type=1&page=1&pagesize=30",
    )?)?;

    let list = json
        .as_array()
        .ok_or_else(|| anyhow!("返回数据不是数组"))?;

    let mut item_vec = Vec::new();
    for article in list {
        let title = article["NewsTitle"].as_str().unwrap_or("");
        let url_path = article["url"].as_str().unwrap_or("");
        let author = article["NewsAuthor"]
            .as_str()
            .filter(|s| !s.is_empty())
            .or_else(|| article["NewsSource"].as_str())
            .or_else(|| article["CreaterName"].as_str())
            .unwrap_or("");
        let create_date = article["CreateDate"].as_str().unwrap_or("");
        let channel = article["ChannelName"].as_str().unwrap_or("");
        let notes = article["NewsNotes"].as_str().unwrap_or("");
        let origin_pic = article["originPic"].as_str().unwrap_or("");

        if title.is_empty() || url_path.is_empty() {
            continue;
        }

        let link = if url_path.starts_with("http") {
            url_path.to_string()
        } else {
            format!("https://www.yicai.com{}", url_path)
        };

        let pub_date = if !create_date.is_empty() {
            let dt = create_date.replace('T', " ");
            datetime_str_to_rss(&dt).unwrap_or_else(now)
        } else {
            now()
        };

        let mut description = String::new();
        if !origin_pic.is_empty() {
            description.push_str(&format!(
                "<figure><img src=\"{}\" alt=\"{}\"></figure>",
                origin_pic, title
            ));
        }
        if !notes.is_empty() {
            description.push_str(&format!("<p>{}</p>", notes));
        }

        match fetch_reqwest_get(&link) {
            Ok(detail_html) => {
                let doc = Html::parse_document(&detail_html);
                for selector in &[".multiText", "#multi-text", ".txt", ".m-txt"] {
                    if let Some(content) = doc
                        .select(&Selector::parse(selector).unwrap())
                        .next()
                    {
                        description.push_str(&content.inner_html());
                        break;
                    }
                }
            }
            Err(_) => {}
        }

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(Some(link))
            .author(Some(author.to_string()))
            .pub_date(pub_date)
            .description(Some(description))
            .categories(vec![CategoryBuilder::default()
                .name(channel.to_string())
                .build()])
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("第一财经 - 头条")
        .link("https://www.yicai.com")
        .description("第一财经头条文章".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
