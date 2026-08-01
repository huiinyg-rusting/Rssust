use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let category = para.get("category").map(|s| s.as_str()).unwrap_or("all");
    let period = para.get("type").map(|s| s.as_str()).unwrap_or("weekly");

    let sort_type: i64 = match period {
        "monthly" => 30,
        "historical" => 0,
        _ => 7,
    };
    let period_title = match period {
        "monthly" => "本月",
        "historical" => "历史",
        _ => "本周",
    };

    let (url, body) = if category == "all" {
        (
            "https://api.juejin.cn/recommend_api/v1/article/recommend_all_feed".to_string(),
            format!(
                r#"{{"cursor":"0","id_type":2,"limit":20,"sort_type":{}}}"#,
                sort_type
            ),
        )
    } else {
        let cat_resp =
            fetch_reqwest_get("https://api.juejin.cn/tag_api/v1/query_category_briefs").await?;
        let cat_json: Value = serde_json::from_str(&cat_resp)?;
        let categories = cat_json["data"]
            .as_array()
            .ok_or_else(|| anyhow!("找不到 categories"))?;
        let cat = categories
            .iter()
            .find(|c| c["category_url"].as_str() == Some(category))
            .ok_or_else(|| anyhow!("找不到分类: {}", category))?;
        let cate_id = cat["category_id"]
            .as_str()
            .ok_or_else(|| anyhow!("category_id 无效"))?;

        (
            "https://api.juejin.cn/recommend_api/v1/article/recommend_cate_feed".to_string(),
            format!(
                r#"{{"cursor":"0","id_type":2,"limit":20,"sort_type":{},"cate_id":"{}"}}"#,
                sort_type, cate_id
            ),
        )
    };

    let resp = fetch_reqwest_post_json(&url, &body).await?;
    let json: Value = serde_json::from_str(&resp)?;
    let items = json["data"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 data 字段"))?;

    let article_items: Vec<&Value> = if category == "all" {
        items
            .iter()
            .filter(|item| item["item_type"].as_i64() == Some(2))
            .map(|item| &item["item_info"])
            .collect()
    } else {
        items.iter().collect()
    };

    let mut item_vec = Vec::new();
    for item in article_items {
        let title = item["article_info"]["title"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let brief = item["article_info"]["brief_content"].as_str().unwrap_or("");
        let ctime = item["article_info"]["ctime"]
            .as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| item["article_info"]["ctime"].as_i64())
            .unwrap_or(0);
        let article_id = item["article_id"].as_str().unwrap_or("");
        let author = item["author_user_info"]["user_name"].as_str().unwrap_or("");

        let link = format!("https://juejin.cn/post/{}", article_id);
        let pub_date = timestamp_to_rss(ctime);

        let mut tags = Vec::new();
        if let Some(tag_list) = item["tags"].as_array() {
            for tag in tag_list {
                if let Some(name) = tag["tag_name"].as_str() {
                    tags.push(name.to_string());
                }
            }
        }
        let category_name = item["category"]["category_name"].as_str().unwrap_or("");
        if !category_name.is_empty() {
            tags.push(category_name.to_string());
        }

        let rss_item = ItemBuilder::default()
            .title(Some(title))
            .link(link)
            .description(Some(brief.to_string()))
            .pub_date(pub_date)
            .author(Some(author.to_string()))
            .categories(
                tags.into_iter()
                    .map(|t| {
                        let mut cat = rss::Category::default();
                        cat.set_name(t);
                        cat
                    })
                    .collect::<Vec<rss::Category>>(),
            )
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("掘金热门 - {}", period_title))
        .link("https://juejin.cn/trending")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
