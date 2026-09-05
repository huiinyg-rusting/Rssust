use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const UA: &str = UA_CHROME;

///GitHub Trending repositories (no official RSS, robots.txt allows /trending).
///Params: since (daily/weekly/monthly, default daily), limit (optional, max 25, default 25)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let since = para
        .get("since")
        .cloned()
        .unwrap_or_else(|| "daily".to_string());
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(25)
        .min(25);

    let url = format!("https://github.com/trending?since={}", since);
    let html = fetch_reqwest_get_with_headers(&url, &[("User-Agent", UA)]).await?;

    let doc = Html::parse_document(&html);
    let box_sel = Selector::parse(r#"article.Box-row"#).unwrap();
    let link_sel = Selector::parse(r#"h2 a[href]"#).unwrap();
    let desc_sel = Selector::parse(r#"p.col-9"#).unwrap();
    let lang_sel = Selector::parse(r#"[itemprop="programmingLanguage"]"#).unwrap();
    let stars_sel = Selector::parse(r#"span.d-inline-block.float-sm-right"#).unwrap();

    let mut item_vec = Vec::new();
    for art in doc.select(&box_sel) {
        if item_vec.len() >= limit {
            break;
        }
        let Some(a) = art.select(&link_sel).next() else {
            continue;
        };
        let Some(href) = a.value().attr("href") else {
            continue;
        };
        let repo = href.trim_start_matches('/');
        if repo.is_empty() || !repo.contains('/') {
            continue;
        }
        let title = a
            .text()
            .collect::<String>()
            .replace('\n', "")
            .replace(' ', "");
        let desc = art
            .select(&desc_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let lang = art
            .select(&lang_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let stars = art
            .select(&stars_sel)
            .next()
            .map(|e| {
                let t = e.text().collect::<String>().trim().to_string();
                t.replace(" stars today", "")
                    .replace(" stars this week", "")
                    .replace(" stars this month", "")
                    .trim()
                    .to_string()
            })
            .unwrap_or_default();

        let link = format!("https://github.com/{}", repo);
        let mut desc_html = String::new();
        if !desc.is_empty() {
            desc_html.push_str(&format!("<p>{}</p>", escape_html(&desc)));
        }
        let mut meta = Vec::new();
        if !lang.is_empty() {
            meta.push(format!("Language: {}", escape_html(&lang)));
        }
        if !stars.is_empty() {
            meta.push(format!("Stars today: {}", escape_html(&stars)));
        }
        if !meta.is_empty() {
            desc_html.push_str(&format!("<p>{}</p>", meta.join("<br>")));
        }

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(link)
                .description(if desc_html.is_empty() {
                    None
                } else {
                    Some(desc_html)
                })
                .pub_date(now())
                .guid(rss::Guid {
                    value: format!("https://github.com/{}#trending", repo),
                    permalink: false,
                })
                .author(Some(repo.split('/').next().unwrap_or("").to_string()))
                .build(),
        );
    }

    if item_vec.is_empty() {
        return Err(anyhow!("No trending repositories found"));
    }

    let channel = ChannelBuilder::default()
        .title(format!("GitHub Trending - {}", since))
        .link(url)
        .description("Trending repositories on GitHub (no official RSS, scraped)")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
