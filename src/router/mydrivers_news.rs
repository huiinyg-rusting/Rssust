use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::{Datelike, Local};
use regex::Regex;
use rss::*;
use std::collections::HashMap;

const PAGE_URL: &str = "https://www.mydrivers.com/";

fn rss_datetime(date: &str) -> String {
    datetime_str_to_rss(date).unwrap_or_else(now)
}

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let body = fetch_reqwest_get_with_headers(PAGE_URL, &[("User-Agent", UA_CHROME)]).await?;
    let re = Regex::new(
        r#"<span class="titl">.*?<a class="[^"]*" href="([^"]+\.htm)"[^>]*>([^<]+)</a></span><span class="([^"]*)"[^>]*>([^<]+)</span>"#,
    )
    .expect("mydrivers 正则编译失败");

    let today = Local::now();
    let mut item_vec = Vec::new();
    for caps in re.captures_iter(&body) {
        let link = resolve_url(&caps[1], PAGE_URL);
        let title = caps[2].trim().to_string();
        if title.is_empty() {
            continue;
        }
        let class = &caps[3];
        let text = caps[4].trim();
        let pub_date = if class.contains("today") && Regex::new(r"^(\d{2}):(\d{2})$").unwrap().is_match(text) {
            let caps2 = Regex::new(r"^(\d{2}):(\d{2})$").unwrap().captures(text).unwrap();
            rss_datetime(&format!(
                "{}-{:02}-{:02} {}:{}:00",
                today.year(),
                today.month(),
                today.day(),
                &caps2[1],
                &caps2[2]
            ))
        } else if let Some(caps2) = Regex::new(r"^(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})$")
            .ok()
            .and_then(|re| re.captures(text))
        {
            rss_datetime(&format!(
                "{}-{}-{} {}:{}:00",
                &caps2[1], &caps2[2], &caps2[3], &caps2[4], &caps2[5]
            ))
        } else if let Some(caps2) = Regex::new(r"^(\d{1,2})日$").ok().and_then(|re| re.captures(text)) {
            rss_datetime(&format!(
                "{}-{:02}-{:02} 00:00:00",
                today.year(),
                today.month(),
                caps2[1].parse::<u32>().unwrap_or(1)
            ))
        } else {
            now()
        };

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title.clone()))
                .link(link.clone())
                .guid(Some(
                    GuidBuilder::default()
                        .value(link.clone())
                        .permalink(true)
                        .build(),
                ))
                .description(Some(format!("<p>【快科技】{}</p>", title)))
                .pub_date(pub_date)
                .build(),
        );
    }
    if item_vec.is_empty() {
        return Err(anyhow!(HttpError::bad_gateway("快科技首页解析失败")));
    }

    let channel = ChannelBuilder::default()
        .title("快科技 - 今日资讯")
        .link(PAGE_URL.to_string())
        .description("快科技科技资讯列表")
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}