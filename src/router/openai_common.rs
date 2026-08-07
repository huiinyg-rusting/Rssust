use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

/// OpenAI 新闻/研究 共用抓取逻辑：
/// 官方 RSS（openai.com/news/rss.xml）+ 详情页补全（cloudflare 挑战，需 curl-impersonate chrome110）
pub const BASE_URL: &str = "https://openai.com";
pub const RSS_URL: &str = "https://openai.com/news/rss.xml";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36";
const PROFILE: &str = "chrome110";

pub fn parse_pub_date(s: &str) -> String {
    chrono::DateTime::parse_from_rfc2822(s)
        .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
        .unwrap_or_else(|_| now())
}

/// 抓取文章详情页，返回 (内容HTML, 分类列表, 作者, 图片)
pub async fn fetch_article_details(url: &str) -> Result<(String, Vec<String>, String, String), Error> {
    let normalized = if url.ends_with('/') {
        url.to_string()
    } else {
        format!("{}/", url)
    };
    let html = fetch_browser_get_with_headers_profile(&normalized, &[("User-Agent", UA)], PROFILE)
        .await?;
    let doc = Html::parse_document(&html);

    let article_sel = Selector::parse("#main article").map_err(|e| anyhow!("{}", e))?;
    let article = doc
        .select(&article_sel)
        .next()
        .ok_or_else(|| anyhow!("article not found"))?;
    let content = article.html();

    let mut categories = Vec::new();
    let cat_sel = Selector::parse("h1 a[href]").map_err(|e| anyhow!("{}", e))?;
    for el in doc.select(&cat_sel) {
        let t = el.text().collect::<String>().trim().to_string();
        if !t.is_empty() {
            categories.push(t);
        }
    }

    let mut authors = Vec::new();
    let auth_sel = Selector::parse("[data-testid=\"author-list\"] a").map_err(|e| anyhow!("{}", e))?;
    for el in doc.select(&auth_sel) {
        let t = el.text().collect::<String>().trim().to_string();
        if !t.is_empty() {
            authors.push(t);
        }
    }

    let mut image = String::new();
    let img_sel = Selector::parse("meta[property=\"og:image\"]").map_err(|e| anyhow!("{}", e))?;
    if let Some(m) = doc.select(&img_sel).next() {
        if let Some(v) = m.value().attr("content") {
            image = v.to_string();
        }
    }

    Ok((content, categories, authors.join(", "), image))
}

/// 拉取官方 RSS，按 category 过滤（None 不过滤），抓取前 limit 条的详情
pub async fn fetch_articles(limit: usize, category: Option<&str>) -> Result<Vec<rss::Item>, Error> {
    let xml = fetch_reqwest_get_with_headers(RSS_URL, &[("User-Agent", UA)]).await?;
    let channel = Channel::read_from(xml.as_bytes())
        .map_err(|e| anyhow!("解析 RSS 失败: {}", e))?;

    let sources: Vec<rss::Item> = channel
        .items
        .iter()
        .filter(|item| {
            if let Some(cat) = category {
                item.categories.iter().any(|c| c.name.trim() == cat)
            } else {
                true
            }
        })
        .take(limit)
        .cloned()
        .collect();

    // 预收集每条元数据，随后并发抓取详情页
    let metas: Vec<(String, String, String, String, Vec<String>, String)> = sources
        .iter()
        .map(|src| {
            (
                src.title.clone().unwrap_or_default(),
                src.link.clone().unwrap_or_default(),
                src.pub_date.clone().unwrap_or_default(),
                src.guid
                    .as_ref()
                    .map(|g| g.value.clone())
                    .unwrap_or_else(|| src.link.clone().unwrap_or_default()),
                src.categories.iter().map(|c| c.name.clone()).collect(),
                src.description.clone().unwrap_or_default(),
            )
        })
        .collect();

    let handles: Vec<_> = metas
        .iter()
        .map(|(_, link, _, _, _, _)| {
            let link = link.clone();
            tokio::spawn(async move { fetch_article_details(&link).await })
        })
        .collect();

    let mut items = Vec::new();
    for ((title, link, pub_date, guid, cats, fallback_desc), handle) in
        metas.into_iter().zip(handles)
    {
        let (mut desc, cats, author, image) = match handle.await {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => {
                tracing::warn!("detail fetch failed for {}: {}", link, e);
                (fallback_desc, cats, String::new(), String::new())
            }
            Err(e) => {
                tracing::warn!("detail fetch join failed for {}: {}", link, e);
                (fallback_desc, cats, String::new(), String::new())
            }
        };

        if !image.is_empty() && !desc.contains("<img") {
            let img_html = format!(
                "<img src=\"{}\" alt=\"\"><br><br>",
                image.replace('&', "&amp;")
            );
            desc = format!("{}{}", img_html, desc);
        }

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(link.clone())
            .description(Some(desc))
            .pub_date(parse_pub_date(&pub_date))
            .guid(if guid.is_empty() {
                rss::Guid {
                    value: link.clone(),
                    permalink: false,
                }
            } else {
                rss::Guid {
                    value: guid,
                    permalink: false,
                }
            })
            .author(if author.is_empty() {
                None
            } else {
                Some(author)
            })
            .categories(
                cats.into_iter()
                    .map(|n| rss::Category {
                        name: n,
                        domain: None,
                    })
                    .collect::<Vec<rss::Category>>(),
            )
            .build();
        items.push(item);
    }
    Ok(items)
}

/// 通用 openai 路由入口参数解析：limit
pub fn parse_limit(para: &HashMap<String, String>) -> usize {
    para.get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10)
        .min(30)
}

/// 解析 help.openai.com 发布说明页（chatgpt / chatgpt-atlas 共用）。
/// 每段以 `.article-content` 内的 h1 分隔；title 取自紧随的第一个 h2（chatgpt）或 h1 本身（atlas）。
pub async fn fetch_release_notes(
    article_url: &str,
    use_h2_title: bool,
) -> Result<(String, Vec<rss::Item>), Error> {
    const PROFILE: &str = "chrome110";
    const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36";
    let html = fetch_browser_get_with_headers_profile(article_url, &[("User-Agent", UA)], PROFILE)
        .await?;
    let doc = Html::parse_document(&html);

    let h1_sel = Selector::parse("h1").map_err(|e| anyhow!("{}", e))?;
    let feed_title = doc
        .select(&h1_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "OpenAI Release Notes".to_string());

    let content_sel = Selector::parse(".article-content").map_err(|e| anyhow!("{}", e))?;
    let content = doc
        .select(&content_sel)
        .next()
        .ok_or_else(|| anyhow!("Failed to find article content"))?;

    let mut section: Vec<(String, String)> = Vec::new(); // (title, description)
    let mut pending_title: Option<String> = None;
    let mut pending_desc = String::new();

    for el in content.descendent_elements() {
        let name = el.value().name();
        if name == "h1" {
            // flush previous
            if let Some(t) = pending_title.take() {
                section.push((t, std::mem::take(&mut pending_desc)));
            }
            pending_title = Some(el.text().collect::<String>().trim().to_string());
        } else if pending_title.is_some() {
            pending_desc.push_str(&el.html());
        }
    }
    if let Some(t) = pending_title.take() {
        section.push((t, pending_desc));
    }

    let mut items = Vec::new();
    for (text, desc) in section {
        let clean = text.replace("**", "").trim().to_string();
        let date_match = regex::Regex::new(r"(\w+\s+\d+[stndrh]*,\s+\d{4})")
            .map_err(|e| anyhow!("{}", e))?
            .captures(&clean);
        let pub_date = date_match
            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
            .and_then(|d| parse_release_date(&d));

        let title = if use_h2_title {
            // 取紧随第一个 h2 的文本
            let h2_text = extract_first_h2_text(&desc);
            if h2_text.is_empty() { clean.clone() } else { h2_text }
        } else {
            clean.clone()
        };

        let guid = format!(
            "{}#{}",
            article_url,
            pub_date.clone().unwrap_or_else(|| clean.clone())
        );

        items.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(article_url.to_string())
                .description(Some(desc))
                .guid(rss::Guid { value: guid, permalink: false })
                .pub_date(pub_date.unwrap_or_else(now))
                .build(),
        );
    }

    Ok((feed_title, items))
}

/// 从 release 段落 HTML 中提取第一个 h2 的文本
fn extract_first_h2_text(html: &str) -> String {
    let frag = Html::parse_fragment(html);
    let h2_sel = Selector::parse("h2").ok();
    if let Some(sel) = h2_sel {
        if let Some(el) = frag.select(&sel).next() {
            return el.text().collect::<String>().trim().to_string();
        }
    }
    String::new()
}

/// "August 4th, 2026" / "July 31, 2026" -> RFC2822 (视为 UTC 当天)
fn parse_release_date(s: &str) -> Option<String> {
    let re = regex::Regex::new(r"(\w+)\s+(\d{1,2})(?:st|nd|rd|th)?,\s+(\d{4})").ok()?;
    let caps = re.captures(s)?;
    let month = caps.get(1)?.as_str();
    let day: u32 = caps.get(2)?.as_str().parse().ok()?;
    let year: i32 = caps.get(3)?.as_str().parse().ok()?;

    let month_map: HashMap<&str, u32> = [
        ("January", 1), ("February", 2), ("March", 3), ("April", 4),
        ("May", 5), ("June", 6), ("July", 7), ("August", 8),
        ("September", 9), ("October", 10), ("November", 11), ("December", 12),
    ]
    .iter()
    .cloned()
    .collect();
    let m = *month_map.get(month)?;

    let fixed = chrono::FixedOffset::east_opt(0)?;
    let naive = chrono::NaiveDate::from_ymd_opt(year, m, day)?.and_hms_opt(0, 0, 0)?;
    let dt: chrono::DateTime<chrono::FixedOffset> =
        chrono::DateTime::from_naive_utc_and_offset(naive, fixed);
    Some(dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
}
