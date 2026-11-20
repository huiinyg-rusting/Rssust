use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::DateTime;
use rss::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

const UA: &str = UA_CHROME;

pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let html = fetch_reqwest_get_with_headers("https://www.defensenews.com", &[("User-Agent", UA)])
        .await?;

    /* 站点 2026 年改版为 WordPress Gutenberg 结构：
    - 文章卡片：<li class="wp-block-post ...">
    - 标题/链接：标题标签在 h3/h4/h5/h6 间浮动，统一按 .wp-block-post-title a 取
    - 摘要：p.wp-block-post-excerpt__excerpt
    - 时间：卡片内 <time datetime="RFC3339">，无需再逐篇抓详情页
    - 配图：img.wp-post-image */
    let doc = Html::parse_document(&html);
    let card_sel = Selector::parse("li.wp-block-post").map_err(|_| anyhow!("card selector"))?;
    let title_sel =
        Selector::parse(".wp-block-post-title a").map_err(|_| anyhow!("title selector"))?;
    let excerpt_sel = Selector::parse(".wp-block-post-excerpt__excerpt")
        .map_err(|_| anyhow!("excerpt selector"))?;
    let date_sel =
        Selector::parse(".wp-block-post-date time").map_err(|_| anyhow!("date selector"))?;
    let img_sel = Selector::parse("img.wp-post-image").map_err(|_| anyhow!("image selector"))?;

    let mut items = Vec::new();
    for card in doc.select(&card_sel) {
        let Some(title_el) = card.select(&title_sel).next() else {
            continue;
        };
        let title = title_el.text().collect::<String>().trim().to_string();
        let Some(href) = title_el.value().attr("href") else {
            continue;
        };
        if title.is_empty() {
            continue;
        }
        let link = if href.starts_with("http") {
            href.to_string()
        } else {
            format!("https://www.defensenews.com{}", href)
        };

        let excerpt = card
            .select(&excerpt_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let image = card
            .select(&img_sel)
            .next()
            .and_then(|e| e.value().attr("src"))
            .unwrap_or_default();
        let pub_date = card
            .select(&date_sel)
            .next()
            .and_then(|e| e.value().attr("datetime"))
            .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
            .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
            .unwrap_or_else(now);

        let mut description = if excerpt.is_empty() {
            "Defense News 最新军事新闻".to_string()
        } else {
            excerpt
        };
        if !image.is_empty() {
            description = format!(
                "<img src=\"{}\" referrerpolicy=\"no-referrer\"><br>{}",
                image, description
            );
        }

        items.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(link)
                .pub_date(pub_date)
                .description(Some(description))
                .build(),
        );
        if items.len() >= 20 {
            break;
        }
    }
    tracing::info!("defensenews: built {} items", items.len());

    let channel = ChannelBuilder::default()
        .title("Defense News - Global Defense & Military News")
        .link("https://www.defensenews.com")
        .description(
            "Defense News latest articles - global defense, military & aerospace (no official RSS)",
        )
        .items(items)
        .build();

    Ok(channel.to_string())
}
