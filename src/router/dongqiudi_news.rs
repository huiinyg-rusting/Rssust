use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let id = opt_usize(&para, "id", 1);

    let json: Value = serde_json::from_str(
        fetch_reqwest_get(&format!(
            "https://api.dongqiudi.com/app/tabs/web/{}.json",
            id
        ))
        .await?
        .as_str(),
    )?;

    let label = json["label"].as_str().unwrap_or("新闻");
    let articles = json["articles"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 articles 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    for art in articles {
        let title = art["title"].as_str().unwrap_or_default();
        let aid = art["id"].as_i64().unwrap_or(0);
        if title.is_empty() || aid == 0 {
            continue;
        }

        let show_time = art["show_time"].as_i64().unwrap_or(0);
        let pub_date = if show_time > 0 {
            timestamp_to_rss(show_time)
        } else {
            now()
        };

        let category = art["category"].as_str().unwrap_or_default();
        let author = art["author_name"].as_str().unwrap_or_default();
        let intro = art["description"].as_str().unwrap_or_default();

        let mut description = String::new();
        if !category.is_empty() {
            description.push_str(&format!("分类: {}<br>", category));
        }
        if !intro.is_empty() {
            description.push_str(&format!("<blockquote>{}</blockquote>", intro));
        }

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(title.to_string())))
            .link(Some(format!(
                "https://www.dongqiudi.com/articles/{}.html",
                aid
            )))
            .description(Some(description))
            .pub_date(pub_date)
            .author(Some(author.to_string()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("懂球帝 - {}", label))
        .link("https://www.dongqiudi.com/".to_string())
        .description("懂球帝新闻".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}