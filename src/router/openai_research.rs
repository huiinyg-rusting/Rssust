use crate::router::openai_common::{fetch_articles, parse_limit};
use anyhow::{Error, Result, anyhow};
use rss::*;
use std::collections::HashMap;

/// OpenAI Research（只收录 category=Research 的文章）
/// 来源：openai.com/news/rss.xml 官方 RSS + 详情页
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let limit = parse_limit(&para);
    let items = fetch_articles(limit, Some("Research")).await?;

    if items.is_empty() {
        return Err(anyhow!("没有抓取到任何 Research 内容"));
    }

    let channel = ChannelBuilder::default()
        .title("OpenAI Research")
        .link("https://openai.com/research/index/")
        .description("OpenAI research papers and announcements")
        .items(items)
        .build();
    Ok(channel.to_string())
}
