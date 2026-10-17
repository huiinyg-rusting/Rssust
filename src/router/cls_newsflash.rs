use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use md5::{Digest, Md5};
use rss::*;
use serde_json::Value;
use sha1::Sha1;
use std::collections::HashMap;

/* 财联社 Web API 的 query 必须先按 key 字典序拼成串，再 sha1 -> md5 作为 sign 参数 */
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

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let root_url = "https://www.cls.cn";
    let api_url = format!("{}/v1/roll/get_roll_list", root_url);

    // category 为空表示全部快讯；rn 为返回条数；last_time 为翻页游标（上一页最后一条的 ctime）
    let category = req_optional(&para, "category").unwrap_or_default();
    // 上游会静默忽略未知 category 并返回全部快讯，这里先校验，避免标题与实际内容不符
    if !category.is_empty()
        && !["watch", "announcement", "jjyw", "hk", "red"].contains(&category.as_str())
    {
        return Err(anyhow!(
            "未知 category \"{}\"，可选值：watch / announcement / jjyw / hk / red，留空表示全部",
            category
        ));
    }
    // 服务端对 rn>50 直接返回空列表，故钳到 50
    let rn = opt_usize(&para, "rn", 20).clamp(1, 50);
    let rn_str = rn.to_string();
    let last_time = req_optional(&para, "last_time").unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
            .to_string()
    });

    let mut params = vec![
        ("app", "CailianpressWeb"),
        ("os", "web"),
        ("sv", "8.4.6"),
        ("category", category.as_str()),
        ("rn", rn_str.as_str()),
        // refresh_type=1 才会按 last_time 向前翻页，否则 last_time 被忽略
        ("refresh_type", "1"),
        ("last_time", last_time.as_str()),
    ];
    let sign = build_sign(&params);
    params.push(("sign", &sign));

    let param_str: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let json: Value =
        serde_json::from_str(&fetch_reqwest_get(&format!("{}?{}", api_url, param_str)).await?)?;

    let errno = json["errno"].as_i64().unwrap_or(-1);
    if errno != 0 {
        return Err(anyhow!(
            "财联社接口返回错误：errno={} msg={}",
            errno,
            json["msg"].as_str().unwrap_or("")
        ));
    }

    let data = json["data"]["roll_data"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 data.roll_data"))?;
    if data.is_empty() {
        return Err(anyhow!(
            "财联社快讯返回空列表，category \"{}\" 或 last_time \"{}\" 可能无效",
            category,
            last_time
        ));
    }

    let mut item_vec = Vec::new();
    for item in data {
        let id = item["id"].as_i64().unwrap_or(0);
        if id == 0 {
            continue;
        }
        let title = item["title"].as_str().unwrap_or("");
        let title = if title.is_empty() {
            item["brief"].as_str().unwrap_or("")
        } else {
            title
        };
        if title.is_empty() {
            continue;
        }

        let content = item["content"].as_str().unwrap_or("");
        let brief = item["brief"].as_str().unwrap_or("");
        let body = if content.is_empty() { brief } else { content };
        let pub_ts = item["ctime"].as_i64().unwrap_or(0);
        let level = item["level"].as_str().unwrap_or("");

        let mut description = String::new();
        if !level.is_empty() {
            description.push_str(&format!("<p>级别: {}</p>", level));
        }
        if !body.is_empty() {
            description.push_str(&format!("<p>{}</p>", body));
        }
        if let Some(stocks) = item["stock_list"].as_array() {
            let names: Vec<String> = stocks
                .iter()
                .map(|s| {
                    let name = s["name"].as_str().unwrap_or("");
                    let code = s["StockID"].as_str().unwrap_or("");
                    let rise = s["RiseRange"].as_f64();
                    match (code.is_empty(), rise) {
                        (true, _) => name.to_string(),
                        (false, Some(r)) => format!("{}({} {:+.2}%)", name, code, r),
                        (false, None) => format!("{}({})", name, code),
                    }
                })
                .filter(|s| !s.is_empty())
                .collect();
            if !names.is_empty() {
                description.push_str(&format!("<p>相关个股：{}</p>", names.join("、")));
            }
        }

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(title.to_string())))
            .link(Some(format!("{}/detail/{}", root_url, id)))
            .description(Some(description))
            .pub_date(timestamp_to_rss(pub_ts))
            .build();
        item_vec.push(item);
    }

    let channel_label = match category.as_str() {
        "" => "全部",
        "watch" => "看盘",
        "announcement" => "公告",
        "jjyw" => "基金要闻",
        "hk" => "港股",
        "red" => "精选",
        other => other,
    };

    let channel = ChannelBuilder::default()
        .title(format!("财联社 - 快讯 ({})", channel_label))
        .link("https://www.cls.cn/telegraph".to_string())
        .description("财联社实时电报快讯".to_string())
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
