use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::DateTime;
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const MAINTAINER: &str = "AI制作 / huinyg-rusting审核";
const UA: &str = "rssust/1.0 (Cargo registry aggregator; +https://github.com/huinyg/Rssust)";

fn version(c: &Value) -> String {
    c["max_stable_version"]
        .as_str()
        .or_else(|| c["max_version"].as_str())
        .or_else(|| c["default_version"].as_str())
        .unwrap_or("")
        .to_string()
}

fn crates_pubdate(created: &str) -> String {
    DateTime::parse_from_rfc3339(created)
        .ok()
        .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
        .unwrap_or_else(now)
}

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let count = para
        .get("count")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10)
        .clamp(1, 100);
    let url = format!("https://crates.io/api/v1/crates?sort=new&per_page={}", count);

    let body = fetch_reqwest_get_with_headers(&url, &[("User-Agent", UA)]).await?;
    let json: Value = serde_json::from_str(body.as_str())?;
    let crates = json["crates"]
        .as_array()
        .ok_or_else(|| anyhow!("crates.io 返回缺少 crates 数组"))?;

    let mut item_vec = Vec::new();
    for c in crates {
        let name = c["name"].as_str().unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }
        let ver = version(c);
        let link = format!("https://crates.io/crates/{}", name);
        let descr = c["description"].as_str().unwrap_or("");
        let downloads = c["downloads"].as_i64().unwrap_or(0);
        let recent = c["recent_downloads"].as_i64().unwrap_or(0);
        let repo = c["repository"].as_str().unwrap_or("");

        let mut desc = String::new();
        if !descr.is_empty() {
            desc.push_str(&format!("<p>{}</p>", descr));
        }
        desc.push_str(&format!(
            "<p>版本 {} · 累计下载 {} · 近 90 天下载 {}</p>",
            ver, downloads, recent
        ));
        if !repo.is_empty() {
            desc.push_str(&format!("<p><a href=\"{}\">仓库</a></p>", repo));
        }

        let title = if ver.is_empty() {
            name
        } else {
            format!("{} v{}", name, ver)
        };
        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(link.clone())
                .guid(Some(
                    GuidBuilder::default()
                        .value(link.clone())
                        .permalink(true)
                        .build(),
                ))
                .description(Some(desc))
                .pub_date(crates_pubdate(c["created_at"].as_str().unwrap_or("")))
                .build(),
        );
    }

    let channel = ChannelBuilder::default()
        .title("crates.io 最新发布的 crate")
        .link("https://crates.io/crates?sort=new")
        .description(format!("crates.io 官方 API 聚合 | {}", MAINTAINER))
        .pub_date(now())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}