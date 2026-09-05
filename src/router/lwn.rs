use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use std::collections::HashMap;

const UA: &str = UA_CHROME;
const MAINTAINER: &str = "AI转写 / huiinyg审核";

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let feed = para
        .get("feed")
        .map(|s| s.as_str())
        .unwrap_or("rss");
    let url = format!("https://lwn.net/headlines/{}", feed);

    let body = fetch_reqwest_get_with_headers(&url, &[("User-Agent", UA)]).await?;
    let channel = Channel::read_from(body.as_bytes())
        .map_err(|e| anyhow!("解析 LWN RSS 失败: {}", e))?;

    let mut item_vec = Vec::new();
    for item in channel.items() {
        let author = item
            .dublin_core_ext()
            .and_then(|dc| dc.creators().first().cloned())
            .unwrap_or_default();
        item_vec.push(
            ItemBuilder::default()
                .title(item.title().map(|s| s.to_string()))
                .link(item.link().map(|s| s.to_string()))
                .guid(item.guid().cloned())
                .description(item.description().map(|s| s.to_string()))
                .pub_date(item.pub_date().map(|s| s.to_string()))
                .author(Some(author))
                .build(),
        );
    }

    let out = ChannelBuilder::default()
        .title(channel.title().to_string())
        .link(channel.link().to_string())
        .description(format!(
            "{} | {}",
            channel.description(),
            MAINTAINER
        ))
        .items(item_vec)
        .build();
    Ok(out.to_string())
}