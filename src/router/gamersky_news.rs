use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use rss::*;
use std::collections::HashMap;

const PAGE_URL: &str = "https://www.gamersky.com/news/";

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let body = fetch_reqwest_get_with_headers(PAGE_URL, &[("User-Agent", UA_CHROME)]).await?;
    let re = Regex::new(
        r#"(?s)class="tt" href="([^"]+)" target="_blank" title="([^"]*)".*?<div class="time">([^<]+)</div>"#,
    )
    .expect("gamersky 正则编译失败");

    let mut item_vec = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for caps in re.captures_iter(&body) {
        let link = resolve_url(&caps[1], PAGE_URL);
        let title = caps[2].trim().to_string();
        if title.is_empty() || !seen.insert(link.clone()) {
            continue;
        }
        let date = &caps[3];
        let pub_date = if Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$")
            .unwrap()
            .is_match(date)
        {
            datetime_str_to_rss(&format!("{}:00", date)).unwrap_or_else(now)
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
                .description(Some(format!("<p>【游民星空】{}</p>", title)))
                .pub_date(pub_date)
                .build(),
        );
    }
    if item_vec.is_empty() {
        return Err(anyhow!(HttpError::bad_gateway("游民星空新闻解析失败")));
    }

    let channel = ChannelBuilder::default()
        .title("游民星空 - 游戏资讯")
        .link(PAGE_URL.to_string())
        .description("游民星空游戏资讯列表")
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}