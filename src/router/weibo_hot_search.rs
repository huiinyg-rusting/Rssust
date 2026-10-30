use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

/// 微博热搜榜（对应 RSSHub /weibo/search/hot）。
/// 上游：https://weibo.com/ajax/side/hotSearch （公开 JSON 接口，无需登录）
/// 参数：
///  - limit：返回条数，默认 50（上游一次返回 50 条）
///  - hotgov：是否包含置顶话题（bool，默认 false）
///
/// 限流：建议在配置 routes.rate_limit["/weibo_hot_search"] = 60，避免触发风控
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let limit = opt_usize(&para, "limit", 50);
    let include_hotgov = parse_bool(para.get("hotgov"), false);

    let resp = fetch_reqwest_get_with_headers(
        "https://weibo.com/ajax/side/hotSearch",
        &[
            ("User-Agent", UA_CHROME),
            ("Referer", "https://weibo.com/"),
            ("Accept", "application/json"),
        ],
    )
    .await?;
    let json: Value = serde_json::from_str(&resp)?;
    if json["ok"].as_i64().unwrap_or(0) != 1 {
        return Err(anyhow!(
            "微博接口返回异常（可能触发风控），响应：{}",
            truncate(&resp, 120)
        ));
    }
    let realtime = json["data"]["realtime"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 data.realtime 字段"))?;

    /* 组 RSS 条目：纯 JSON 数据，无 await 后残留引用问题 */
    let mut item_vec = Vec::new();

    if include_hotgov && let Some(gov) = json["data"]["hotgov"].as_object() {
        let word = strip_hashtag(gov["word"].as_str().unwrap_or(""));
        let note = strip_hashtag(gov["note"].as_str().unwrap_or(""));
        if !word.is_empty()
            && let Some(g) = build_item(&word, &note, None, 0, None)
        {
            item_vec.push(g);
        }
    }

    for entry in realtime {
        /* 广告条目跳过（正常榜单无 is_ad，出现在投广告时用） */
        if entry["is_ad"].as_i64().unwrap_or(0) == 1 {
            continue;
        }
        let word = strip_hashtag(entry["word"].as_str().unwrap_or(""));
        if word.is_empty() {
            continue;
        }
        let note = entry["note"].as_str().unwrap_or("");
        let rank = entry["realpos"].as_i64().unwrap_or(0);
        let num = entry["num"].as_i64().unwrap_or(0);
        let label = entry["label_name"].as_str().unwrap_or("");
        let label = if label.is_empty() {
            None
        } else {
            Some(label.to_string())
        };

        if let Some(item) = build_item(&word, note, label.as_deref(), rank, Some(num)) {
            item_vec.push(item);
        }
        if item_vec.len() >= limit {
            break;
        }
    }

    if item_vec.is_empty() {
        return Err(anyhow!("没有抓取到任何热搜条目"));
    }

    let channel = ChannelBuilder::default()
        .title(format!("微博热搜榜（前{}条）", item_vec.len()))
        .link("https://s.weibo.com/top/summary")
        .description("微博热搜榜")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

/* 去掉词条首尾的 # 号（word 字段可能带也可能不带） */
fn strip_hashtag(s: &str) -> String {
    s.trim_matches('#').to_string()
}

/* 把一条热搜转成 RSS Item：
word 搜索词（不带#），title_note 标题候选（note 通常与 word 相同），
label 榜单标签（如“爆”“热”），rank 榜位，num 热度值。
word 为空或无有效链接时返回 None。 */
fn build_item(
    word: &str,
    title_note: &str,
    label: Option<&str>,
    rank: i64,
    num: Option<i64>,
) -> Option<Item> {
    let title = if title_note.is_empty() {
        word.to_string()
    } else {
        title_note.to_string()
    };

    /* 搜索页链接：s.weibo.com/weibo?q=%23词条%23（词条保留 # 才等价于话题搜索） */
    let scheme = format!("#{}#", word.trim_matches('#'));
    let link = format!(
        "https://s.weibo.com/weibo?q={}",
        urlencoding::encode(&scheme)
    );

    /* 描述：榜位 + 标签 + 热度 */
    let mut desc = if rank > 0 {
        format!("第 {} 位", rank)
    } else {
        String::from("置顶")
    };
    if let Some(l) = label {
        desc.push_str(&format!(" · {}", l));
    }
    if let Some(n) = num {
        desc.push_str(&format!(" · 热度 {}", n));
    }
    desc.push_str(&format!("<br><a href=\"{}\">查看微博讨论</a>", link));

    Some(
        ItemBuilder::default()
            .title(Some(title))
            .link(Some(link))
            .description(Some(desc))
            .pub_date(now())
            .guid(rss::Guid {
                value: format!("weibo-hot-{}", word),
                permalink: false,
            })
            .build(),
    )
}
