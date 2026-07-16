use crate::easyuser::*;
use anyhow::{Error, Ok, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get(_para:HashMap<String, String>) -> Result<String,Error>{
    let json: Value = serde_json::from_str(fetch_reqwest_get("https://api.zhihu.com/topstory/hot-lists/total?limit=10&reverse_order=0")?.as_str())?;
    let mut item_vec = Vec::new();
    for ts in json.pointer("/data")
        .and_then(|list| list.as_array())
        .ok_or_else(|| anyhow!("找不到 data.archives 字段或不是数组"))? {
            let question_id = ts["target"]["url"].to_string();
            let final_link = format!("https://www.zhihu.com/question/{}", question_id);
            let pub_time = timestamp_to_rss(ts["target"]["created"].as_i64().ok_or_else(|| anyhow!("知乎的api缺少 时间 字段"))?);
            let item = ItemBuilder::default()
            .title(Some(no_double_quotes(ts["target"]["title"].to_string())))
            .link(final_link)
            .description(Some(no_double_quotes(ts["target"]["excerpt"].to_string())))
            .pub_date(pub_time)
            .build();
        item_vec.push(item);
        };
        let channel = ChannelBuilder::default()
        .title("知乎热榜")
        .link("https://www.zhihu.com/hot")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}