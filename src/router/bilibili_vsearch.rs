use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

fn strip_html_tags(s: &str) -> String {
    let re = Regex::new(r"<[^>]*>").unwrap();
    let cleaned = re.replace_all(s, "");
    cleaned
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
}

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let kw = para
        .get("kw")
        .ok_or_else(|| anyhow!("缺少 kw 参数"))?;
    let order = para.get("order").map(|s| s.as_str()).unwrap_or("pubdate");
    let tid = para.get("tid").map(|s| s.as_str()).unwrap_or("0");

    let kw_url_encoded = urlencoding::encode(kw);
    let url = format!(
        "https://api.bilibili.com/x/web-interface/search/type?search_type=video&highlight=1&keyword={}&order={}&tids={}",
        kw_url_encoded, order, tid
    );
    let referer = format!("https://search.bilibili.com/all?keyword={}", kw_url_encoded);

    let cookie = load_cookie_header(Some("bilibili.com")).ok().flatten();
    let mut headers: Vec<(&str, &str)> = vec![("Referer", referer.as_str())];
    if let Some(c) = &cookie {
        headers.push(("Cookie", c));
    }

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &headers)?.as_str(),
    )?;

    let result = json
        .pointer("/data/result")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data/result 字段或不是数组"))?;

    let mut item_vec = Vec::new();

    for item in result {
        let title = item["title"].as_str().unwrap_or_default();
        let author = item["author"].as_str().unwrap_or_default();
        let mid = item["mid"].as_i64().unwrap_or(0);
        let pic = item["pic"].as_str().unwrap_or_default();
        let description_text = item["description"].as_str().unwrap_or_default();
        let arcurl = item["arcurl"].as_str().unwrap_or_default();
        let duration = item["duration"].as_str().unwrap_or_default();
        let play = item["play"].as_i64().unwrap_or(0);
        let favorites = item["favorites"].as_i64().unwrap_or(0);
        let video_review = item["video_review"].as_i64().unwrap_or(0);
        let review = item["review"].as_i64().unwrap_or(0);
        let tag = item["tag"].as_str().unwrap_or_default();
        let typename = item["typename"].as_str().unwrap_or_default();
        let pubdate_val = item["pubdate"].as_i64().unwrap_or(0);
        let aid = item["aid"].as_i64();
        let bvid = item["bvid"].as_str();

        let title_clean = strip_html_tags(title);
        let des_clean = description_text.replace('\n', "<br/>");
        let img_url = if pic.starts_with("//") {
            format!("http:{}", pic)
        } else {
            pic.to_string()
        };

        let mut categories: Vec<rss::Category> = Vec::new();
        if !tag.is_empty() {
            for s in tag.split(',') {
                let mut cat = rss::Category::default();
                cat.set_name(s.to_string());
                categories.push(cat);
            }
        }
        if !typename.is_empty() {
            let mut cat = rss::Category::default();
            cat.set_name(typename.to_string());
            categories.push(cat);
        }

        let iframe = match para.get("disableembed") {
            Some(s) if s == "false" => String::new(),
            _ => {
                if let (Some(a), Some(b)) = (aid, bvid) {
                    format!(
                        r#"<iframe width="640" height="360" src="https://www.bilibili.com/blackboard/html5mobileplayer.html?aid={}&amp;cid=undefined&amp;bvid={}" frameborder="0" allowfullscreen="" referrerpolicy="no-referrer"></iframe>"#,
                        a, b
                    )
                } else {
                    String::new()
                }
            }
        };

        let description = format!(
            "Length: {}<br/>AuthorID: {}<br/>Play: {}    Favorite: {}<br/>Danmaku: {}    Comment: {}<br/><br/>{}<br/><img src=\"{}\" referrerpolicy=\"no-referrer\"><br/>{}",
            duration, mid, play, favorites, video_review, review, des_clean, img_url, iframe
        );

        let item = ItemBuilder::default()
            .title(Some(title_clean))
            .link(Some(arcurl.to_string()))
            .description(description)
            .author(Some(author.to_string()))
            .pub_date(timestamp_to_rss(pubdate_val))
            .categories(categories)
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("{} - bilibili", kw))
        .link(format!(
            "https://search.bilibili.com/all?keyword={}&order={}",
            kw_url_encoded, order
        ))
        .description(format!(
            "Result from {} bilibili search, ordered by {}.",
            kw, order
        ))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
