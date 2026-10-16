use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let channel = req_optional(&para, "channel").unwrap_or_else(|| "all".to_string());

    // timestamp 为毫秒时间戳，接口要求带上（服务端用它做缓存 key）
    // 注意：接口的 limit 参数服务端会忽略，返回条数由频道决定（all=15，popular=50）
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let json: Value = serde_json::from_str(
        &fetch_reqwest_get(&format!(
            "https://www.gelonghui.com/api/live-channels/{}/lives/v4?category={}&limit=20&timestamp={}",
            channel, channel, timestamp
        ))
        .await?,
    )?;

    let result = json["result"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 result 字段"))?;
    if result.is_empty() {
        return Err(anyhow!("格隆汇快讯返回空列表，频道 \"{}\" 可能不存在", channel));
    }

    let mut item_vec = Vec::new();
    for entry in result {
        let id = entry["id"].as_i64().unwrap_or(0);
        let title = entry["title"].as_str().unwrap_or("");
        let content = entry["content"].as_str().unwrap_or("");
        let ts = entry["createTimestamp"].as_i64().unwrap_or(0);
        let link = entry["route"].as_str().unwrap_or("");

        if id == 0 || title.is_empty() {
            continue;
        }
        let link = if link.is_empty() {
            format!("https://www.gelonghui.com/live/{}", id)
        } else {
            link.to_string()
        };

        let mut description = String::new();
        if !content.is_empty() {
            description.push_str(&format!("<p>{}</p>", content));
        }
        // 关联个股以文字列出，便于阅读器直接看到涉及的标的
        if let Some(stocks) = entry["relatedStocks"].as_array() {
            let names: Vec<String> = stocks
                .iter()
                .filter_map(|s| {
                    let market = s["market"].as_str().unwrap_or("");
                    let code = s["code"].as_str().unwrap_or("");
                    let name = s["name"].as_str().unwrap_or("");
                    if name.is_empty() {
                        return None;
                    }
                    if code.is_empty() {
                        Some(name.to_string())
                    } else {
                        Some(format!("{}({}{})", name, market, code))
                    }
                })
                .collect();
            if !names.is_empty() {
                description.push_str(&format!("<p>相关个股：{}</p>", names.join("、")));
            }
        }

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(title.to_string())))
            .link(Some(link))
            .description(Some(description))
            .pub_date(timestamp_to_rss(ts))
            .build();
        item_vec.push(item);
    }

    let channel_label = channel_label(&channel);
    let channel = ChannelBuilder::default()
        .title(format!("格隆汇 - 快讯 ({})", channel_label))
        .link("https://www.gelonghui.com/live/".to_string())
        .description("格隆汇实时快讯".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

fn channel_label(channel: &str) -> &str {
    match channel {
        "all" => "全部",
        "popular" => "最热",
        "international" => "国际",
        "ai" => "AI",
        "HKStock" => "港股",
        "virtualAssets" => "虚拟资产",
        "chineseMainland" => "中国大陆",
        "AStock" => "A股",
        "hongkongMacaoTaiwan" => "港澳台",
        "USStock" => "美股",
        "exchangeCommodity" => "商品外汇",
        "house" => "楼市",
        "fundLive" => "基金",
        "debenture" => "债券",
        _ => channel,
    }
}
