use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

/// QQ 音乐排行榜。
/// 上游：https://u.y.qq.com/cgi-bin/musicu.fcg 网页端公开 API（GetAll + GetDetail），
/// u.y.qq.com 无 robots.txt（404 视为允许），QQ音乐也没有官方 RSS。
/// 两步请求：
///   1. GetAll(type=2) 拉全部榜单元信息（标题/期号 period/更新时间）
///   2. GetDetail 按榜单 id + period 拉歌曲明细
/// 参数：
///  - id：榜单 topId，默认 26（热歌榜）；常用：62 飙升榜 / 27 新歌榜 / 4 流行指数榜 /
///        5 内地榜 / 3 欧美榜 / 16 韩国榜 / 17 日本榜 等，完整清单见 GetAll 接口
///  - limit：返回条数，默认 30，最大 100（榜单本身约 100~300 首）
///
/// 限流：建议配置 routes.rate_limit["/qq_music_rank"] = 600
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let id = opt_usize(&para, "id", 26);
    let limit = opt_usize(&para, "limit", 30).min(100);

    /* 第一步：枚举榜单，拿到该 id 的期号 period 与更新时间 */
    let all = musicu(r#"{ "top": { "module": "musicToplist.ToplistInfoServer", "method": "GetAll", "param": { "type": 2 } } }"#).await?;
    let groups = all["top"]["data"]["group"]
        .as_array()
        .ok_or_else(|| anyhow!(" GetAll 返回缺少 top.data.group"))?;
    let mut meta: Option<Value> = None;
    'outer: for group in groups {
        for top in group["toplist"]
            .as_array()
            .map(|v| v.as_slice())
            .unwrap_or(&[])
        {
            if top["topId"].as_u64() == Some(id as u64) {
                meta = Some(top.clone());
                break 'outer;
            }
        }
    }
    let meta = meta.ok_or_else(|| {
        anyhow!(
            "找不到 topId={} 的榜单（常用：26 热歌榜 / 62 飙升榜 / 27 新歌榜 / 4 流行指数榜）",
            id
        )
    })?;
    let title = meta["title"].as_str().unwrap_or_default().to_string();
    let period = meta["period"].as_str().unwrap_or_default().to_string();
    let update_time = meta["updateTime"].as_str().unwrap_or(&period).to_string();
    if period.is_empty() {
        return Err(anyhow!("榜单 {} 没有期号 period", title));
    }

    /* 第二步：拉取榜单歌曲明细 */
    let payload = serde_json::json!({
        "topList": {
            "module": "musicToplist.ToplistInfoServer",
            "method": "GetDetail",
            "param": { "topId": id, "offset": 0, "num": limit, "period": period }
        }
    })
    .to_string();
    let detail = musicu(&payload).await?;
    let data = &detail["topList"]["data"];
    if detail["topList"]["code"].as_i64().unwrap_or(-1) != 0 {
        return Err(anyhow!(
            "GetDetail 失败：code={}",
            detail["topList"]["code"]
        ));
    }
    let songs = data["songInfoList"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 songInfoList 字段"))?;

    let mut item_vec = Vec::new();
    for (idx, song) in songs.iter().enumerate() {
        let name = song["name"].as_str().unwrap_or("").trim().to_string();
        if name.is_empty() {
            continue;
        }
        let singers = song["singer"]
            .as_array()
            .map_or(Vec::new(), |v| {
                v.iter()
                    .filter_map(Value::as_object)
                    .filter_map(|o| o.get("name"))
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .join(" / ");
        let album = song["album"]["name"].as_str().unwrap_or("").to_string();
        let interval = song["interval"].as_i64().unwrap_or(0);
        let mid = song["mid"].as_str().unwrap_or("");
        if mid.is_empty() {
            continue;
        }
        let link = format!("https://y.qq.com/n/ryqq/songDetail/{}", mid);
        let album_html = if album.is_empty() {
            String::new()
        } else {
            format!("专辑《{}》，", album)
        };
        let description = format!(
            "<b>{}</b>{}时长 {}<br><a href=\"{}\">在 QQ 音乐查看</a>",
            name,
            album_html,
            mmss(interval),
            link
        );

        item_vec.push(
            ItemBuilder::default()
                .title(Some(format!("{}. {} - {}", idx + 1, name, singers)))
                .link(Some(link))
                .description(Some(description))
                .pub_date(date_str_to_rss(&update_time, "%Y-%m-%d", "+08:00").unwrap_or_else(now))
                .guid(rss::Guid {
                    value: format!("qq-music-top{}-{}-{}", id, period, idx + 1),
                    permalink: false,
                })
                .build(),
        );
        if item_vec.len() >= limit {
            break;
        }
    }

    if item_vec.is_empty() {
        return Err(anyhow!(
            "榜单「{}」（期号 {}）没有解析到歌曲",
            title,
            period
        ));
    }

    let channel = ChannelBuilder::default()
        .title(format!("QQ 音乐 {}（期号 {}）", title, period))
        .link(format!("https://y.qq.com/n/ryqq/toplist/{}", id))
        .description(format!("QQ音乐排行榜 · 榜单期号 {}", period))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

/* musicu.fcg 通用调用：入参为紧凑 JSON 字符串 */
async fn musicu(data: &str) -> Result<Value, Error> {
    let url = format!(
        "https://u.y.qq.com/cgi-bin/musicu.fcg?data={}",
        urlencoding::encode(data)
    );
    let resp = fetch_reqwest_get_with_headers(
        &url,
        &[
            ("User-Agent", UA_CHROME),
            ("Accept", "application/json"),
            ("Referer", "https://y.qq.com/"),
        ],
    )
    .await?;
    let json: Value =
        serde_json::from_str(&resp).map_err(|e| anyhow!("QQ音乐返回非 JSON：{}", e))?;
    if json["code"].as_i64().unwrap_or(-1) != 0 {
        return Err(anyhow!("musicu.fcg 顶层 code={}", json["code"]));
    }
    Ok(json)
}

/* 秒数 → mm:ss（超过 1 小时则为 h:mm:ss） */
fn mmss(secs: i64) -> String {
    if secs <= 0 {
        return "未知".to_string();
    }
    let (h, rest) = (secs / 3600, secs % 3600);
    if h > 0 {
        format!("{}:{:02}:{:02}", h, rest / 60, rest % 60)
    } else {
        format!("{:02}:{:02}", rest / 60, rest % 60)
    }
}
