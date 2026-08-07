use crate::router::openai_common::fetch_release_notes;
use anyhow::{Error, Result, anyhow};
use rss::*;
use std::collections::HashMap;

/// OpenAI ChatGPT Atlas Release Notes
/// 来源：help.openai.com 官方发布说明单页
pub async fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    const ARTICLE_URL: &str = "https://help.openai.com/en/articles/12591856-chatgpt-atlas-release-notes";

    let (feed_title, items) = fetch_release_notes(ARTICLE_URL, false).await?;

    if items.is_empty() {
        return Err(anyhow!("没有抓取到任何内容"));
    }

    let channel = ChannelBuilder::default()
        .title(format!("{} - ChatGPT Atlas Release Notes", feed_title))
        .link(ARTICLE_URL)
        .description("ChatGPT Atlas Release Notes")
        .items(items)
        .build();
    Ok(channel.to_string())
}
