use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use md5::{Digest, Md5};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use sha1::Sha1;
use std::collections::HashMap;

fn build_sign(params: &[(&str, &str)]) -> String {
    let mut sorted = params.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(b.0));
    let query: String = sorted
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let mut sha1 = Sha1::new();
    sha1.update(query.as_bytes());
    let sha1_hex = hex::encode(sha1.finalize());

    let mut md5 = Md5::new();
    md5.update(sha1_hex.as_bytes());
    hex::encode(md5.finalize())
}

pub fn get(_para: HashMap<String, String>) -> Result<String, Error> {
    let root_url = "https://www.cls.cn";
    let api_url = format!("{}/v2/article/hot/list", root_url);

    let mut params = vec![
        ("appName", "CailianpressWeb"),
        ("os", "web"),
        ("sv", "8.7.9"),
    ];
    let sign = build_sign(&params);
    params.push(("sign", &sign));

    let param_str: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let url = format!("{}?{}", api_url, param_str);

    let json: Value = serde_json::from_str(&fetch_reqwest_get(&url)?)?;

    let data = json["data"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 data"))?;

    let mut item_vec = Vec::new();
    for item in data {
        let title = item["title"]
            .as_str()
            .or_else(|| item["brief"].as_str())
            .unwrap_or("");
        let id = item["id"].as_i64().unwrap_or(0);
        let link = format!("{}/detail/{}", root_url, id);
        let pub_ts = item["ctime"].as_i64().unwrap_or(0);
        let pub_date = timestamp_to_rss(pub_ts);
        let author = item["author"].as_str().unwrap_or("").to_string();

        if title.is_empty() {
            continue;
        }

        let description = match fetch_reqwest_get(&link) {
            Ok(detail_html) => {
                let detail_doc = Html::parse_document(&detail_html);
                let sel = Selector::parse("script#__NEXT_DATA__")
                    .map_err(|_| anyhow!("选择器无效"))?;
                if let Some(el) = detail_doc.select(&sel).next() {
                    let json_str: String = el.text().collect();
                    if let Ok(next_data) =
                        serde_json::from_str::<Value>(&json_str)
                    {
                        let article = &next_data["props"]["pageProps"]["articleDetail"];
                        let content = article["content"]
                            .as_str()
                            .unwrap_or("");
                        let images = article["images"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();

                        let mut desc = String::new();
                        for img in &images {
                            desc.push_str(&format!(
                                r#"<img src="{}"><br>"#,
                                img
                            ));
                        }
                        desc.push_str(content);
                        desc
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }
            Err(_) => String::new(),
        };

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(link)
            .pub_date(pub_date)
            .description(Some(description))
            .author(Some(author))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title("财联社 - 热门文章排行榜")
        .link(root_url.to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
