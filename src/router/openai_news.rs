use crate::router::openai_common::{fetch_articles, parse_limit};
use anyhow::{Error, Result, anyhow};
use rss::*;
use std::collections::HashMap;

/// OpenAI News
/// 来源：openai.com/news/rss.xml 官方 RSS + 详情页
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let limit = parse_limit(&para);
    let items = fetch_articles(limit, None).await?;

    if items.is_empty() {
        return Err(anyhow!("没有抓取到任何内容"));
    }

    let channel = ChannelBuilder::default()
        .title("OpenAI News")
        .link("https://openai.com/news/")
        .description("The OpenAI blog")
        .items(items)
        .build();
    Ok(channel.to_string())
}
