use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use reqwest::header::{HeaderMap, HeaderValue};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    // 分区 id 可能是负数（如水区 -7），opt_usize 解析不了，这里用 i64
    let fid = match para.get("fid") {
        Some(v) => v.parse::<i64>().unwrap_or(489),
        None => 489,
    };

    // 移动端接口要一张"访客票"cookie，时间戳新鲜即可，不用登录
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow!("系统时间异常: {}", e))?
        .as_secs();

    let mut headers = HeaderMap::new();
    headers.insert(
        "X-User-Agent",
        HeaderValue::from_static("NGA_skull/6.0.5(iPhone10,3;iOS 12.0.1)"),
    );
    headers.insert(
        reqwest::header::COOKIE,
        HeaderValue::from_str(&format!("guestJs={};", ts))
            .map_err(|e| anyhow!("guestJs 头构建失败: {}", e))?,
    );
    // 不带这个头服务端不认表单，会回 code=4 参数错误
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );

    let json: Value = serde_json::from_str(
        fetch_reqwest_post(
            "https://ngabbs.com/app_api.php?__lib=subject&__act=list",
            format!("fid={}", fid),
            Some(headers),
        )
        .await?
        .as_str(),
    )?;

    if json["code"].as_i64() != Some(0) {
        return Err(anyhow!(
            "NGA接口返回错误 code={}: {}",
            json["code"].as_i64().unwrap_or(-1),
            json["msg"].as_str().unwrap_or("")
        ));
    }

    let forumname = json["forumname"].as_str().unwrap_or("NGA");
    let list = json
        .pointer("/result/data")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("找不到 result.data 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    for t in list {
        let tid = t["tid"].as_i64().unwrap_or(0);
        let subject = t["subject"].as_str().unwrap_or_default();
        if tid == 0 || subject.is_empty() {
            continue;
        }

        let author = t["author"].as_str().unwrap_or_default();
        let replies = t["replies"].as_i64().unwrap_or(0);
        let postdate = t["postdate"].as_i64().unwrap_or(0);
        let pub_date = if postdate > 0 {
            timestamp_to_rss(postdate)
        } else {
            now()
        };

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(subject.to_string())))
            .link(Some(format!("https://nga.178.com/read.php?tid={}", tid)))
            .description(Some(format!("楼主: {}<br>回复数: {}", author, replies)))
            .pub_date(pub_date)
            .author(Some(author.to_string()))
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("NGA - {}", forumname))
        .link(format!("https://nga.178.com/thread.php?fid={}", fid))
        .description("NGA玩家社区分区帖子".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
