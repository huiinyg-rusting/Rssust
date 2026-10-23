use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

/* AcFun 排行榜：https://www.acfun.cn/rest/pc-direct/rank/channel
- rankPeriod 仅支持 DAY / THREE_DAYS / WEEK（MONTH 上游返回 result=21）
- rankLimit 真实生效，接口无翻页；单次上限按 100 收敛
- channelId 固定 0（全站），AcFun 未暴露可稳定枚举的频道列表 */
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let period = req_optional(&para, "period")
        .unwrap_or_else(|| "day".to_string())
        .to_lowercase();
    let (rank_period, period_label) = match period.as_str() {
        "day" => ("DAY", "日榜"),
        "three_days" | "three-days" | "3days" => ("THREE_DAYS", "三日榜"),
        "week" => ("WEEK", "周榜"),
        other => {
            return Err(anyhow!(
                "未知 period \"{}\"，可选值：day / three_days / week（AcFun 月榜上游不支持）",
                other
            ));
        }
    };
    let limit = opt_usize(&para, "limit", 30).clamp(1, 100);

    let api_url = format!(
        "https://www.acfun.cn/rest/pc-direct/rank/channel?channelId=0&rankPeriod={}&rankLimit={}",
        rank_period, limit
    );
    let json: Value = serde_json::from_str(&fetch_reqwest_get(&api_url).await?)?;

    let result = json["result"].as_i64().unwrap_or(-1);
    if result != 0 {
        return Err(anyhow!(
            "AcFun 排行榜接口返回错误：result={}，周期 \"{}\" 暂不可用",
            result,
            period
        ));
    }

    let list = json["rankList"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 rankList 字段或不是数组"))?;
    if list.is_empty() {
        return Err(anyhow!(
            "AcFun 排行榜返回空列表，周期 \"{}\" 可能暂无数据",
            period
        ));
    }

    let mut item_vec = Vec::new();
    for entry in list {
        let content_id = entry["contentId"].as_i64().unwrap_or(0);
        if content_id == 0 {
            continue;
        }

        let title = entry["contentTitle"]
            .as_str()
            .filter(|s| !s.is_empty())
            .or_else(|| entry["title"].as_str())
            .unwrap_or("");
        if title.is_empty() {
            continue;
        }

        let desc = entry["contentDesc"]
            .as_str()
            .filter(|s| !s.is_empty())
            .or_else(|| entry["description"].as_str())
            .unwrap_or("");
        let user_name = entry["userName"]
            .as_str()
            .filter(|s| !s.is_empty())
            .or_else(|| entry["user"]["name"].as_str())
            .unwrap_or("");
        let cover = entry["videoCover"]
            .as_str()
            .filter(|s| !s.is_empty())
            .or_else(|| entry["coverUrl"].as_str())
            .unwrap_or("");
        let ts_ms = entry["createTimeMillis"].as_i64().unwrap_or(0);
        let channel_name = entry["channel"]["name"]
            .as_str()
            .filter(|s| !s.is_empty())
            .or_else(|| entry["channelName"].as_str())
            .unwrap_or("");
        let parent_name = entry["channel"]["parentName"].as_str().unwrap_or("");
        let view = entry["viewCount"].as_i64().unwrap_or(0);
        let like = entry["likeCount"].as_i64().unwrap_or(0);
        let comment = entry["commentCount"].as_i64().unwrap_or(0);
        let danmaku = entry["danmakuCount"]
            .as_i64()
            .or_else(|| entry["danmuCount"].as_i64())
            .unwrap_or(0);
        let duration_ms = entry["durationMillis"].as_i64().unwrap_or(0);

        let mut description = String::new();
        if !cover.is_empty() {
            description.push_str(&format!(
                r#"<img src="{}" referrerpolicy="no-referrer"><br>"#,
                cover
            ));
        }
        if !desc.is_empty() {
            description.push_str(&format!("<p>{}</p>", desc));
        }

        let mut meta = Vec::new();
        if !user_name.is_empty() {
            meta.push(format!("UP主：{}", user_name));
        }
        if !channel_name.is_empty() {
            if parent_name.is_empty() {
                meta.push(format!("频道：{}", channel_name));
            } else {
                meta.push(format!("频道：{}/{}", parent_name, channel_name));
            }
        }
        if !meta.is_empty() {
            description.push_str(&format!("<p>{}</p>", meta.join(" | ")));
        }

        let mut stats = vec![format!("播放：{}", view)];
        if like > 0 {
            stats.push(format!("点赞：{}", like));
        }
        if comment > 0 {
            stats.push(format!("评论：{}", comment));
        }
        if danmaku > 0 {
            stats.push(format!("弹幕：{}", danmaku));
        }
        if duration_ms > 0 {
            let total = duration_ms / 1000;
            stats.push(format!("时长：{:02}:{:02}", total / 60, total % 60));
        }
        description.push_str(&format!("<p>{}</p>", stats.join(" | ")));

        if let Some(tags) = entry["tagList"].as_array() {
            let names: Vec<&str> = tags
                .iter()
                .filter_map(|t| t["name"].as_str())
                .filter(|s| !s.is_empty())
                .collect();
            if !names.is_empty() {
                description.push_str(&format!("<p>标签：{}</p>", names.join("、")));
            }
        }

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(title.to_string())))
            .link(Some(format!("https://www.acfun.cn/v/ac{}", content_id)))
            .description(Some(description))
            .pub_date(timestamp_to_rss(ts_ms / 1000))
            .author(if user_name.is_empty() {
                None
            } else {
                Some(user_name.to_string())
            })
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("AcFun 排行榜 - {}", period_label))
        .link("https://www.acfun.cn/rank/list".to_string())
        .description(format!("AcFun {}热门视频排行", period_label))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
