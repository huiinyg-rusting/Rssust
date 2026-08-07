use anyhow::{Error, Result, anyhow};
use rss::{Channel, ChannelBuilder, ItemBuilder};
use std::collections::HashMap;

const RSS_URL: &str = "https://dev.to/feed/tag/guides";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

///DEV.to Trending Guides
///来源：dev.to 官方 RSS（tag=guides）
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10)
        .min(30);

    let xml = crate::easyuser::fetch_reqwest_get_with_headers(RSS_URL, &[("User-Agent", UA)]).await?;
    let channel = Channel::read_from(xml.as_bytes())
        .map_err(|e| anyhow!("解析 RSS 失败: {}", e))?;

    let mut item_vec = Vec::new();
    for src in channel.items.iter().take(limit) {
        let title = src.title.clone().unwrap_or_default();
        let link = src.link.clone().unwrap_or_default();
        let pub_date = src.pub_date.clone().unwrap_or_default();
        let desc = src.description.clone().unwrap_or_default();
        let author = src.author.clone().unwrap_or_default();
        let guid = src
            .guid
            .as_ref()
            .map(|g| g.value.clone())
            .unwrap_or_else(|| link.clone());

        if title.is_empty() || link.is_empty() {
            continue;
        }

        let link_for_guid = link.clone();
        let guid_for_item = guid.clone();

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(link)
            .description(Some(desc))
            .pub_date(crate::router::openai_common::parse_pub_date(&pub_date))
            .guid(if guid.is_empty() {
                rss::Guid {
                    value: link_for_guid,
                    permalink: false,
                }
            } else {
                rss::Guid {
                    value: guid_for_item,
                    permalink: false,
                }
            })
            .author(if author.is_empty() { None } else { Some(author) })
            .categories(
                src.categories
                    .iter()
                    .map(|c| rss::Category {
                        name: c.name.clone(),
                        domain: None,
                    })
                    .collect::<Vec<rss::Category>>(),
            )
            .build();
        item_vec.push(item);
    }

    if item_vec.is_empty() {
        return Err(anyhow!("没有抓取到任何指南"));
    }

    let channel = ChannelBuilder::default()
        .title("DEV.to - Guides")
        .link("https://dev.to/t/guides")
        .description("Latest guides from DEV Community")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}