use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

/// 抖音热搜榜。
/// 上游：https://www.douyin.com/aweme/v1/web/hot/search/list/ 网页端公开 JSON，
/// 该路径不在 robots.txt 的 Disallow 列（douyin 未设置 `User-agent: *` 组），抖音也没有官方 RSS。
/// 返回当前热搜词约 50 条，含热度值、榜单标签关联封面。
/// 参数：
///  - limit：返回条数，默认 10，最大 50
///
/// 限流：建议配置 routes.rate_limit["/douyin_hot"] = 300
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let limit = opt_usize(&para, "limit", 10).min(50);

    let url = "https://www.douyin.com/aweme/v1/web/hot/search/list/?publish_time=7&web_id=6817683000120595463";
    let resp = fetch_reqwest_get_with_headers(
        url,
        &[
            ("User-Agent", UA_CHROME),
            ("Accept", "application/json"),
            ("Referer", "https://www.douyin.com/"),
        ],
    )
    .await?;
    let json: Value = serde_json::from_str(&resp).map_err(|e| anyhow!("抖音返回非 JSON：{}", e))?;

    let word_list = json["data"]["word_list"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 data.word_list 字段（可能被风控，稍后重试）"))?;
    if word_list.is_empty() {
        return Err(anyhow!("抖音热搜榜为空"));
    }

    let mut item_vec = Vec::new();
    for entry in word_list {
        let word = entry["word"].as_str().unwrap_or("").trim().to_string();
        if word.is_empty() {
            continue;
        }
        let rank = entry["position"].as_i64().unwrap_or(0);
        let hot = entry["hot_value"].as_i64().unwrap_or(0);
        let event_time = entry["event_time"].as_i64().unwrap_or(0);
        let cover = entry["word_cover"]["url_list"]
            .as_array()
            .and_then(|v| v.first())
            .and_then(Value::as_str)
            .unwrap_or("");

        let cover_html = if cover.is_empty() {
            String::new()
        } else {
            format!("<img src=\"{}\"><br>", cover)
        };
        let description = format!(
            "{}<b>热度值 {}</b>（排名第 {} 位）<br><a href=\"{}\">在抖音搜索「{}」</a>",
            cover_html,
            thousands(hot),
            rank,
            search_link(&word),
            word
        );

        item_vec.push(
            ItemBuilder::default()
                .title(Some(word.clone()))
                .link(Some(search_link(&word)))
                .description(Some(description))
                .pub_date(if event_time > 0 {
                    timestamp_to_rss(event_time)
                } else {
                    now()
                })
                .guid(rss::Guid {
                    value: format!("douyin-hot-{}-{}", rank, word),
                    permalink: false,
                })
                .build(),
        );
        if item_vec.len() >= limit {
            break;
        }
    }

    if item_vec.is_empty() {
        return Err(anyhow!("未能解析出热搜词"));
    }

    let channel = ChannelBuilder::default()
        .title("抖音热搜榜")
        .link("https://www.douyin.com/hot")
        .description("抖音实时热搜榜（public API）")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

/* 热搜词 → 抖音站内搜索链接 */
fn search_link(word: &str) -> String {
    format!(
        "https://www.douyin.com/search/{}?from=hot_search",
        urlencoding::encode(word)
    )
}

/* 整数加千位分隔符 */
fn thousands(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{}", out) } else { out }
}
