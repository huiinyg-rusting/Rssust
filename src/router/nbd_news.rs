use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use rss::*;
use std::collections::HashMap;

const PAGE_URL: &str = "https://www.nbd.com.cn/";

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let body = fetch_reqwest_get_with_headers(PAGE_URL, &[("User-Agent", UA_CHROME)]).await?;
    let re = Regex::new(
        r#"<a href="https://www\.nbd\.com\.cn/articles/(\d{4}-\d{2}-\d{2})/(\d+)\.html"[^>]*>([^<]+)</a>"#,
    )
    .expect("nbd 正则编译失败");

    let mut item_vec = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for caps in re.captures_iter(&body) {
        let date = &caps[1];
        let id = &caps[2];
        let link = format!("https://www.nbd.com.cn/articles/{}/{}.html", date, id);
        let title = caps[3].trim().to_string();
        if title.is_empty() || !seen.insert(link.clone()) {
            continue;
        }
        let pub_date = datetime_str_to_rss(&format!("{} 00:00:00", date)).unwrap_or_else(now);

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
                .description(Some(format!("<p>【每经网】{}</p>", title)))
                .pub_date(pub_date)
                .build(),
        );
    }
    if item_vec.is_empty() {
        return Err(anyhow!(HttpError::bad_gateway("每经网首页解析失败")));
    }

    let channel = ChannelBuilder::default()
        .title("每经网 - 今日要闻")
        .link(PAGE_URL.to_string())
        .description("每日经济新闻要闻列表")
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}