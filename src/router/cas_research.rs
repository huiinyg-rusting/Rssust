use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use rss::*;
use std::collections::HashMap;

const PAGE_URL: &str = "https://www.cas.cn/syky/";

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let body = fetch_reqwest_get_with_headers(PAGE_URL, &[("User-Agent", UA_CHROME)]).await?;
    let re = Regex::new(
        r#"<li><a href="([^"]+)" title="[^"]*" target="_blank"\s*>(.*?)</a><span>([0-9]{4})年([0-9]{2})月([0-9]{2})日\s*</span></li>"#,
    )
    .expect("cas_research 正则编译失败");
    let date_re = Regex::new(r"t([0-9]{8})_").expect("cas_research 日期正则编译失败");

    let mut item_vec = Vec::new();
    for caps in re.captures_iter(&body) {
        let href = caps.get(1).unwrap().as_str();
        let title = caps.get(2).unwrap().as_str().trim().to_string();
        if title.is_empty() {
            continue;
        }
        // 优先从链接文件名 t20260922_xxx.shtml 提取真实发布日期；
        // 页面 span 里的日期是滚动刷新的当前日期，不可靠。
        let date = match date_re.captures(href) {
            Some(d) => {
                let s = &d[1];
                format!("{}-{}-{} 00:00:00", &s[0..4], &s[4..6], &s[6..8])
            }
            None => format!(
                "{}-{}-{} 00:00:00",
                caps.get(3).unwrap().as_str(),
                caps.get(4).unwrap().as_str(),
                caps.get(5).unwrap().as_str()
            ),
        };
        let link = resolve_url(href, PAGE_URL);
        let pub_date = datetime_str_to_rss(&date).unwrap_or_else(now);

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
                .description(Some("<p>中国科学院科研进展</p>".to_string()))
                .pub_date(pub_date)
                .build(),
        );
    }
    if item_vec.is_empty() {
        return Err(anyhow!(HttpError::bad_gateway(
            "中科院科研进展页面解析失败"
        )));
    }

    let channel = ChannelBuilder::default()
        .title("中国科学院 - 科研进展")
        .link(PAGE_URL.to_string())
        .description("中国科学院科研进展列表")
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
