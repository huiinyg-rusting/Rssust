use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

///B站 UP 主粉丝列表，需 cookies.json 中的 bilibili.com Cookie（登录态）。
///Params: vmid(目标用户mid), pn(页码), limit(每页条数, 默认20, 最大20)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let vmid = para
        .get("vmid")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("缺少 vmid 参数，请在 /bilibili_followers?vmid=xxx 中传入目标用户 mid"))?;
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(20);
    let pn = para
        .get("pn")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);

    let cookie_header = load_cookie_header(Some("bilibili.com")).ok().flatten();
    if cookie_header.is_none() {
        return Err(anyhow!(
            "UP主粉丝列表需要登录态，请先配置 cookies.json (bilibili.com) 中的 Cookie"
        ));
    }

    let referer = format!("https://space.bilibili.com/{}/", vmid);
    let ua = UA_CHROME;
    let cookie = cookie_header.unwrap_or_default();
    let headers: Vec<(&str, &str)> = vec![
        ("Referer", referer.as_str()),
        ("User-Agent", ua),
        ("Cookie", cookie.as_str()),
    ];

    let stat_url = format!("https://api.bilibili.com/x/relation/stat?vmid={}", vmid);
    let count_json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&stat_url, &headers)
            .await?
            .as_str(),
    )?;
    let follower_count = count_json
        .pointer("/data/follower")
        .and_then(Value::as_i64)
        .unwrap_or(0);

    let list_url = format!(
        "https://api.bilibili.com/x/relation/followers?vmid={}&pn={}&ps={}",
        vmid, pn, limit
    );
    let resp = fetch_reqwest_get_with_headers(&list_url, &headers).await?;
    let json: Value = match serde_json::from_str(&resp) {
        Ok(v) => v,
        Err(e) => {
            if resp.contains("412") || resp.contains("security") {
                return Err(anyhow!(
                    "B站风控，请检查 cookies.json (bilibili.com) 的 Cookie 是否有效"
                ));
            }
            return Err(anyhow!(
                "JSON解析失败: {} — 响应片段: {}",
                e,
                &resp.chars().take(200).collect::<String>()
            ));
        }
    };
    if json.pointer("/code").and_then(Value::as_i64) != Some(0) {
        return Err(anyhow!(
            "B站API返回错误 code={}: {}",
            json.pointer("/code").and_then(Value::as_i64).unwrap_or(-1),
            json.pointer("/message")
                .and_then(Value::as_str)
                .unwrap_or("")
        ));
    }

    let list = json
        .pointer("/data/list")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("找不到 data.list 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    for f in list {
        let mid = f["mid"].as_i64().unwrap_or(0);
        let uname = f["uname"].as_str().unwrap_or_default();
        let sign = f["sign"].as_str().unwrap_or_default();
        let mtime = f["mtime"].as_i64().unwrap_or(0);

        let description = format!(
            "{}{}<br>当前粉丝总数: {}",
            uname,
            if sign.is_empty() {
                String::new()
            } else {
                format!("<br>签名: {}", sign)
            },
            follower_count
        );

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(format!("{} 新粉丝 {}", vmid, uname))))
            .link(Some(format!("https://space.bilibili.com/{}", mid)))
            .description(description)
            .pub_date(timestamp_to_rss(mtime))
            .author(Some(uname.to_string()))
            .guid(rss::Guid {
                value: format!("followers-{}-{}-{}", vmid, mid, mtime),
                permalink: false,
            })
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("UID {} 的 bilibili 粉丝", vmid))
        .link(format!("https://space.bilibili.com/{}/fans/fans", vmid))
        .description(format!("UID {} 的 bilibili 粉丝，当前粉丝总数 {}", vmid, follower_count))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}