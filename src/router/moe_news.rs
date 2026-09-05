use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use rss::*;
use std::collections::HashMap;

const MAINTAINER: &str = "AI制作 / huiinyg-rusting审核";
const PAGE_URL: &str = "http://www.moe.gov.cn/jyb_xwfb/gzdt_gzdt/moe_1485/";

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let body = fetch_reqwest_get_with_headers(PAGE_URL, &[("User-Agent", UA_CHROME)]).await?;
    let re = Regex::new(
        r#"<a href="([^"]+)"[^>]*>([^<]+)</a>\s*<span>([0-9]{4}-[0-9]{2}-[0-9]{2})</span>"#,
    )
    .expect("moe_news 正则编译失败");

    let mut item_vec = Vec::new();
    for caps in re.captures_iter(&body) {
        let href = caps.get(1).unwrap().as_str();
        let title = caps.get(2).unwrap().as_str().trim().to_string();
        if title.is_empty() {
            continue;
        }
        let date = caps.get(3).unwrap().as_str();
        let link = resolve_url(href, PAGE_URL);
        let pub_date = datetime_str_to_rss(&format!("{} 00:00:00", date)).unwrap_or_else(now);

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(link.clone())
                .guid(Some(
                    GuidBuilder::default()
                        .value(link.clone())
                        .permalink(true)
                        .build(),
                ))
                .description(Some(format!("<p>教育部要闻 · {}</p>", date)))
                .pub_date(pub_date)
                .build(),
        );
    }
    if item_vec.is_empty() {
        return Err(anyhow!(HttpError::bad_gateway("教育部页面解析失败")));
    }

    let channel = ChannelBuilder::default()
        .title("教育部 - 要闻")
        .link(PAGE_URL.to_string())
        .description(format!("教育部司局要闻列表 | {}", MAINTAINER))
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}