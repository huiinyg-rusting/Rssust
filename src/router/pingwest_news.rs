use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const HOME: &str = "https://www.pingwest.com/";
const UA: &str = UA_CHROME;

///PingWest (品玩) homepage articles (no official RSS, robots.txt allows). Server-side rendered headline / hot-recommend / featured sections.
///Params: limit (optional, max 20, default 14)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(14)
        .min(20);

    let html = fetch_reqwest_get_with_headers(HOME, &[("User-Agent", UA)]).await?;
    let doc = Html::parse_document(&html);

    let mut items: Vec<(String, String, String, String)> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    // hot-recommend: <section class="news-info"> ... <span class="name">author</span> ... <span class="time">date</span>
    // processed first so author/date are preserved (generic pass below dedups)
    let info_sel = Selector::parse(r#"section.news-info"#).unwrap();
    let title_sel = Selector::parse(r#"a.title"#).unwrap();
    let author_sel = Selector::parse(r#"span.name"#).unwrap();
    let time_sel = Selector::parse(r#"span.time"#).unwrap();
    let re = Regex::new(r"^//www\.pingwest\.com/[aw]/\d+$").unwrap();

    for info in doc.select(&info_sel) {
        let Some(a) = info.select(&title_sel).next() else {
            continue;
        };
        let Some(href) = a.value().attr("href") else {
            continue;
        };
        if !re.is_match(href) {
            continue;
        }
        let id = href.rsplit('/').next().unwrap_or("").to_string();
        if seen.contains(&id) {
            continue;
        }
        seen.insert(id.clone());
        let title = a.text().collect::<String>().trim().to_string();
        let author = info
            .select(&author_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let date = info
            .select(&time_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let link = format!("https:{}", href);
        items.push((title, link, author, date));
    }

    // headline carousel + featured: <a href="//www.pingwest.com/{a,w}/{id}" title="..."> or text content
    let a_sel =
        Selector::parse(r#"a[href^="//www.pingwest.com/a/"], a[href^="//www.pingwest.com/w/"]"#)
            .unwrap();
    for a in doc.select(&a_sel) {
        let Some(href) = a.value().attr("href") else {
            continue;
        };
        if !href.contains('/') {
            continue;
        }
        let id = href.rsplit('/').next().unwrap_or("").to_string();
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        if seen.contains(&id) {
            continue;
        }
        let title = a
            .value()
            .attr("title")
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| a.text().collect::<String>().trim().to_string());
        if title.is_empty() {
            continue;
        }
        seen.insert(id.clone());
        let link = format!("https:{}", href);
        items.push((title, link, String::new(), String::new()));
    }

    if items.is_empty() {
        return Err(anyhow!("No articles found on pingwest homepage"));
    }
    items.truncate(limit);

    let mut item_vec = Vec::new();
    for (title, link, author, date) in items {
        let mut desc = String::new();
        if !date.is_empty() {
            desc.push_str(&format!("Published: {}", date));
        }
        if !author.is_empty() {
            if !desc.is_empty() {
                desc.push_str(" | ");
            }
            desc.push_str(&format!("Author: {}", author));
        }
        let pub_date = if date.ends_with("前") {
            now()
        } else {
            chinese_date_to_parse(&date).unwrap_or_else(now)
        };

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(link.clone())
                .description(if desc.is_empty() { None } else { Some(desc) })
                .pub_date(pub_date)
                .guid(rss::Guid {
                    value: format!("{}#pingwest", link),
                    permalink: false,
                })
                .author(if author.is_empty() {
                    None
                } else {
                    Some(author)
                })
                .build(),
        );
    }

    let channel = ChannelBuilder::default()
        .title("品玩 PingWest - 首页资讯")
        .link(HOME)
        .description("品玩首页精选资讯（无官方 RSS，抓取首页静态区块）")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
