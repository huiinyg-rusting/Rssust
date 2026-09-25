use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::{Datelike, Duration, Local};
use regex::Regex;
use rss::*;
use std::collections::HashMap;

const PAGE_URL: &str = "https://www.jiemian.com/";

fn parse_date(raw: &str) -> String {
    if let Some(caps) = Regex::new(r"^(\d{1,2})/(\d{1,2}) (\d{1,2}):(\d{2})$")
        .ok()
        .and_then(|re| re.captures(raw))
    {
        let now_dt = Local::now();
        let date = format!(
            "{}-{:02}-{:02} {}:{}:00",
            now_dt.year(),
            caps[1].parse::<u32>().unwrap_or(1),
            caps[2].parse::<u32>().unwrap_or(1),
            &caps[3],
            &caps[4]
        );
        return datetime_str_to_rss(&date).unwrap_or_else(now);
    }
    let now_value = Local::now();
    for (re_pat, unit) in [
        (r"^(\d+)分钟前$", 60i64),
        (r"^(\d+)小时前$", 3600),
        (r"^(\d+)天前$", 86400),
    ] {
        if let Some(caps) = Regex::new(re_pat).ok().and_then(|re| re.captures(raw)) {
            let n = caps[1].parse::<i64>().unwrap_or(0);
            let dt = now_value - Duration::seconds(n * unit);
            let date = dt.format("%Y-%m-%d %H:%M:%S").to_string();
            return datetime_str_to_rss(&date).unwrap_or_else(now);
        }
    }
    now()
}

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let body = fetch_reqwest_get_with_headers(PAGE_URL, &[("User-Agent", UA_CHROME)]).await?;
    let block_re = Regex::new(r#"(?s)class="news-view">(.*?)</div>[\s]*</div>"#)
        .expect("jiemian block 正则编译失败");
    let id_re = Regex::new(r#"article/(\d+)\.html"#).expect("jiemian id 正则编译失败");
    let title_re =
        Regex::new(r#"class="title[^"]*">([^<]+)</(?:span|p)>"#).expect("jiemian title 正则编译失败");
    let date_re =
        Regex::new(r#"class="date">([^<]+)</span>"#).expect("jiemian date 正则编译失败");

    let mut item_vec = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for block in block_re.captures_iter(&body) {
        let block = &block[1];
        let (Some(id_cap), Some(title_cap), Some(date_cap)) = (
            id_re.captures(block),
            title_re.captures(block),
            date_re.captures(block),
        ) else {
            continue;
        };
        let link = format!("https://www.jiemian.com/article/{}.html", &id_cap[1]);
        let title = title_cap[1].trim().to_string();
        if title.is_empty() || !seen.insert(link.clone()) {
            continue;
        }
        let pub_date = parse_date(&date_cap[1]);

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
                .description(Some(format!("<p>【界面新闻】{}</p>", title)))
                .pub_date(pub_date)
                .build(),
        );
    }
    if item_vec.is_empty() {
        return Err(anyhow!(HttpError::bad_gateway("界面新闻首页解析失败")));
    }

    let channel = ChannelBuilder::default()
        .title("界面新闻 - 要闻")
        .link(PAGE_URL.to_string())
        .description("界面新闻要闻列表")
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}