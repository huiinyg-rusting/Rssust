use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::*;
use std::collections::HashMap;

/// Rust 官方博客最新文章（对应 RSSHub /rust/blog）。
/// 上游：https://blog.rust-lang.org/ 首页的完整文章列表（纯 HTML 表格，含全部历史文章）。
/// 参数：
///  - limit：返回条数，默认 20，最大 100
///
/// 注意：上游有原生 feed.xml，这里的目标是按 RSSHub 同名路由的口径自行转换。
/// 限流：建议配置 routes.rate_limit["/rust_blog"] = 600（遵守 robots Crawl-delay: 10）
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let limit = opt_usize(&para, "limit", 20).min(100);

    let html = fetch_reqwest_get("https://blog.rust-lang.org/").await?;

    /* 第一步：解析列表页，只收集纯数据（title/url/date/year），
    scraper 类型在块结束就释放，后面没有 await，不会有 not Send 问题 */
    let posts: Vec<Post> = {
        let doc = Html::parse_document(&html);
        let link_sel = Selector::parse("td.bn > a").unwrap();
        let date_sel = Selector::parse("td.tr").unwrap();

        doc.select(&link_sel)
            .filter_map(|a| {
                let href = a.value().attr("href")?;
                let title = a.text().collect::<String>().trim().to_string();
                if title.is_empty() {
                    return None;
                }
                /* 行首兄弟单元格 td.tr 是发布日期（形如 "Oct.\u{a0}20"），
                年份直接从链接 /2026/10/20/... 里取，避免解析“Posts in 2026”标题 */
                let date_text = a
                    .parent()
                    .and_then(|td| td.parent())
                    .and_then(ElementRef::wrap)
                    .filter(|tr| tr.value().name() == "tr")
                    .and_then(|tr| tr.select(&date_sel).next())
                    .map(|td| td.text().collect::<String>())
                    .unwrap_or_default();
                let year = href
                    .split('/')
                    .find(|seg| seg.len() == 4 && seg.chars().all(|c| c.is_ascii_digit()))
                    .unwrap_or("")
                    .to_string();
                Some(Post {
                    title,
                    href: href.to_string(),
                    date_text: normalize_date(&date_text),
                    year,
                })
            })
            .take(limit)
            .collect()
    };
    /* 释放 doc：块到这里已经结束 */

    if posts.is_empty() {
        return Err(anyhow!("没有抓取到任何文章，博客页面结构可能已变化"));
    }

    let mut item_vec = Vec::new();
    for post in &posts {
        let link = resolve_url(&post.href, "https://blog.rust-lang.org/");
        let pub_date = post.pub_date();
        item_vec.push(
            ItemBuilder::default()
                .title(Some(post.title.clone()))
                .link(Some(link.clone()))
                .description(Some(format!(
                    "{}<br><a href=\"{}\">阅读原文</a>",
                    pub_date, link
                )))
                .pub_date(pub_date.clone())
                .guid(rss::Guid {
                    value: link,
                    permalink: true,
                })
                .build(),
        );
    }

    let channel = ChannelBuilder::default()
        .title("The Rust Blog")
        .link("https://blog.rust-lang.org/")
        .description("Rust 博客最新文章")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

struct Post {
    title: String,
    href: String,
    /* 归一化后的月日，形如 "Oct 20"；解析失败为空 */
    date_text: String,
    /* 从链接中取的年份，形如 "2026"；取不到为空 */
    year: String,
}

impl Post {
    /* "Oct 20" + "2026" → Sat, 20 Oct 2026 00:00:00 -0500（博客为美东时间） */
    fn pub_date(&self) -> String {
        date_str_to_rss(
            &format!("{} {}", self.year, self.date_text),
            "%Y %b %d",
            "-0500",
        )
        .unwrap_or_else(now)
    }
}

/* "Oct.\u{a0}20" → "Oct 20"；解析失败返回空串 */
fn normalize_date(raw: &str) -> String {
    let cleaned = raw
        .trim()
        .replace(['.', '\u{a0}'], " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");
    /* 校验月份缩写，防止混入其它文本 */
    let month = cleaned.split(' ').next().unwrap_or("");
    if normalize_month(month).is_empty() {
        return String::new();
    }
    cleaned
}

/* "Oct"/"Oct."/"oct" → "Oct"；非法输入返回空串 */
fn normalize_month(m: &str) -> String {
    const MONTHS: &[&str] = &[
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let lower = m.to_lowercase().replace('.', "");
    for name in MONTHS {
        if name.to_lowercase() == lower {
            return (*name).to_string();
        }
    }
    String::new()
}
