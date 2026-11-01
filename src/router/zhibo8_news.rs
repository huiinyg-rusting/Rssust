use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::*;
use std::collections::HashMap;
use std::collections::HashSet;

/// 直播吧新闻（对应 RSSHub 的 8 系列体育资讯路由思路）。
/// 上游：https://news.zhibo8.com/{channel}/ 纯 HTML 列表（无官方 RSS，robots.txt 为 404 无限制）。
/// 列表聚合了多个栏目，同一篇新闻会重复出现，这里按链接去重后取前 limit 条。
/// 参数：
///  - channel：栏目，默认 nba，可选 zuqiu（足球）/ dianjing（电竞）/ other
///  - limit：返回条数，默认 20，最大 50
///  - detail：是否抓详情页正文，默认 true（关闭可显著加快速度，描述只剩标题）
///
/// 限流：建议配置 routes.rate_limit["/zhibo8_news"] = 300
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let channel = para
        .get("channel")
        .cloned()
        .unwrap_or_else(|| "nba".to_string());
    let limit = opt_usize(&para, "limit", 20).min(50);
    let want_detail = parse_bool(para.get("detail"), true);

    let channel_url = format!("https://news.zhibo8.com/{}/", urlencoding::encode(&channel));
    let html = fetch_reqwest_get_with_headers(&channel_url, &[("User-Agent", UA_CHROME)]).await?;

    /* 第一步：解析列表，只留纯数据。scraper 类型在块内用完即释放 */
    let posts: Vec<(String, String)> = {
        let doc = Html::parse_document(&html);
        let sel = Selector::parse("li > a[href*=\"news.zhibo8.com\"]").unwrap();
        let mut seen = HashSet::new();
        doc.select(&sel)
            .filter_map(|a| {
                let href = a.value().attr("href")?;
                /* 列表里既有绝对链接也有 // 开头的协议相对链接 */
                let url = if href.starts_with("//") {
                    format!("https:{}", href)
                } else {
                    resolve_url(href, "https://news.zhibo8.com/")
                };
                let title = a.text().collect::<String>().trim().to_string();
                if title.is_empty() || !has_date_segment(&url) || !seen.insert(url.clone()) {
                    return None;
                }
                Some((title, url))
            })
            .take(limit)
            .collect()
    };

    if posts.is_empty() {
        return Err(anyhow!("没有抓取到任何新闻，栏目 {} 可能不存在", channel));
    }

    let mut item_vec = Vec::new();
    for (title, url) in &posts {
        /* 详情页：正文 + 精确发布时间；失败则退回列表信息 */
        let mut description = String::new();
        let mut pub_date = url_date_to_rss(url);

        if want_detail
            && let Ok(detail_html) =
                fetch_reqwest_get_with_headers(url, &[("User-Agent", UA_CHROME)]).await
        {
            let parsed = {
                let doc = Html::parse_document(&detail_html);
                let content_sel = Selector::parse("#signals").unwrap();
                let time_sel = Selector::parse("span[ms-controller=\"title_controller\"]").unwrap();

                let content = doc
                    .select(&content_sel)
                    .next()
                    .map(|node| node.html())
                    .unwrap_or_default();
                let time_text = doc
                    .select(&time_sel)
                    .next()
                    .map(|node| node.text().collect::<String>())
                    .unwrap_or_default();
                (content, time_text)
            };

            if !parsed.0.is_empty() {
                /* 正文里的图片是 //tu.duoduocdn.com/... 协议相对地址，补全协议 */
                description = strip_script_style(&parsed.0).replace("src=\"//", "src=\"https://");
            }
            if let Some(rss_time) = parse_detail_time(&parsed.1) {
                pub_date = rss_time;
            }
        }

        if description.is_empty() {
            description = format!("<a href=\"{}\">{}</a>", url, title);
        }

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title.clone()))
                .link(Some(url.clone()))
                .description(Some(description))
                .pub_date(pub_date)
                .guid(rss::Guid {
                    value: url.clone(),
                    permalink: true,
                })
                .build(),
        );
    }

    let channel_name = channel_display_name(&channel);
    let channel = ChannelBuilder::default()
        .title(format!("直播吧 - {}", channel_name))
        .link(format!("https://news.zhibo8.com/{}/", channel))
        .description(format!("直播吧{}资讯", channel_name))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

/* 详情页时间文本形如 "2026-10-31 08:55:53 {{info|html}} {{advert|html}}"，取前 19 位解析 */
fn parse_detail_time(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let head: String = trimmed.chars().take(19).collect();
    datetime_str_to_rss(&head)
}

/* 兜底时间：从链接里的 /2026-10-31/ 提取日期 */
fn url_date_to_rss(url: &str) -> String {
    let date = url
        .split('/')
        .find(|seg| is_date_segment(seg))
        .unwrap_or("");
    if date.is_empty() {
        return now();
    }
    date_str_to_rss(date, "%Y-%m-%d", "+0800").unwrap_or_else(now)
}

/* 判断链接里是否有 YYYY-MM-DD 段：用它区分新闻正文页和导航栏链接 */
fn has_date_segment(url: &str) -> bool {
    url.split('/').any(is_date_segment)
}

fn is_date_segment(seg: &str) -> bool {
    let b = seg.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(|c| c.is_ascii_digit())
        && b[5..7].iter().all(|c| c.is_ascii_digit())
        && b[8..].iter().all(|c| c.is_ascii_digit())
}

fn channel_display_name(channel: &str) -> &str {
    match channel {
        "nba" => "NBA",
        "zuqiu" => "足球",
        "dianjing" => "电竞",
        "other" => "综合",
        other => other,
    }
}
