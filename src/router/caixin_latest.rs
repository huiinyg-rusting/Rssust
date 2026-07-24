use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let json: Value = serde_json::from_str(&fetch_reqwest_get(
        "https://gateway.caixin.com/api/dataplatform/scroll/index",
    )?)?;

    let list = json["data"]["articleList"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 articleList"))?;

    let mut item_vec = Vec::new();
    for article in list {
        let title = article["title"].as_str().unwrap_or("");
        let link = article["url"].as_str().unwrap_or("");
        let pub_ts = article["time"].as_i64().unwrap_or(0);
        let category = article["channelObject"]["name"]
            .as_str()
            .unwrap_or("");

        if title.is_empty() || link.is_empty() {
            continue;
        }

        let pub_date = timestamp_to_rss(pub_ts);

        let description = match fetch_reqwest_get(link) {
            Ok(detail_html) => {
                let doc = Html::parse_document(&detail_html);
                let mut desc = String::new();

                if let Some(subhead) = doc
                    .select(&Selector::parse(".article .subhead").unwrap())
                    .next()
                {
                    desc.push_str(&format!(
                        "<blockquote>{}</blockquote><br>",
                        subhead.inner_html()
                    ));
                }

                if let Some(main) = doc
                    .select(&Selector::parse("div#Main_Content_Val.text").unwrap())
                    .next()
                {
                    desc.push_str(&main.inner_html());
                } else {
                    if let Some(summary) = article["summary"].as_str() {
                        if !summary.is_empty() {
                            desc.push_str(&format!("<blockquote>{}</blockquote><br>", summary));
                        }
                    }
                    if let Some(pics) = article["pics"].as_str() {
                        if !pics.is_empty() {
                            for pic in pics.split('#') {
                                desc.push_str(&format!("<img src=\"{}\"><br>", pic));
                            }
                        }
                    }
                }

                desc
            }
            Err(_) => String::new(),
        };

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(Some(link.to_string()))
            .pub_date(pub_date)
            .description(Some(description))
            .categories(vec![CategoryBuilder::default()
                .name(category.to_string())
                .build()])
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("财新网 - 最新文章")
        .link("https://www.caixin.com")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
