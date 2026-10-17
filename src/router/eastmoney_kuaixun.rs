use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

/* 上游 fastColumn 栏目 ID，取自 kuaixun.eastmoney.com 页面配置并逐条实测内容 */
const COLUMNS: &[(&str, &str)] = &[
    ("101", "焦点"),
    ("102", "全球财经"),
    ("103", "上市公司"),
    ("104", "A股"),
    ("107", "外汇"),
    ("108", "债券"),
    ("109", "基金"),
    ("110", "地区-中国"),
    ("111", "地区-美国"),
    ("112", "地区-欧元区"),
    ("113", "地区-英国"),
    ("114", "地区-日本"),
    ("115", "地区-加拿大"),
    ("116", "地区-澳洲"),
    ("117", "地区-新兴市场"),
    ("118", "央行-中国央行"),
    ("119", "央行-美联储"),
    ("120", "央行-欧洲央行"),
    ("121", "央行-英国央行"),
    ("122", "央行-日本央行"),
    ("123", "央行-加拿大央行"),
    ("124", "央行-澳洲联储"),
    ("125", "数据-中国"),
    ("126", "数据-美国"),
    ("127", "数据-欧元区"),
    ("128", "数据-英国"),
    ("129", "数据-日本"),
    ("130", "数据-加拿大"),
    ("131", "数据-澳洲"),
];

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    // column 留空表示全球财经（102），未知取值直接报错，避免与标题内容不符
    let column = req_optional(&para, "column").unwrap_or_default();
    let column = if column.is_empty() {
        "102"
    } else {
        column.as_str()
    };
    let column_name = COLUMNS
        .iter()
        .find(|(id, _)| *id == column)
        .map(|(_, name)| *name)
        .ok_or_else(|| {
            let ids: Vec<&str> = COLUMNS.iter().map(|(id, _)| *id).collect();
            anyhow!(
                "未知 column \"{}\"，可选值：{}，留空表示全球财经",
                column,
                ids.join(" / ")
            )
        })?;

    // 上游 pageSize>200 只返回 200 条，故钳到 200
    let limit = opt_usize(&para, "limit", 20).clamp(1, 200);
    // 翻页游标：传上一页最后一条的 realSort 取更早的快讯
    let sort_end = req_optional(&para, "sort_end").unwrap_or_default();

    let url = format!(
        "https://np-listapi.eastmoney.com/comm/web/getFastNewsList?client=web&biz=web_724&fastColumn={}&sortEnd={}&pageSize={}&req_trace=1",
        column, sort_end, limit
    );

    let json_str =
        fetch_reqwest_get_with_headers(&url, &[("Referer", "https://finance.eastmoney.com/")])
            .await?;
    let json: Value = serde_json::from_str(&json_str)?;

    let list = json
        .pointer("/data/fastNewsList")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data/fastNewsList 字段或不是数组"))?;
    if list.is_empty() {
        return Err(anyhow!(
            "东方财富快讯返回空列表，column \"{}\" 或 sort_end \"{}\" 可能无效",
            column,
            sort_end
        ));
    }

    let mut item_vec = Vec::new();
    for item in list {
        let title = item["title"].as_str().unwrap_or("");
        let summary = item["summary"].as_str().unwrap_or("");
        let code = item["code"].as_str().unwrap_or("");
        let show_time = item["showTime"].as_str().unwrap_or("");

        if title.is_empty() || code.is_empty() {
            continue;
        }

        let link = format!("https://finance.eastmoney.com/a/{}.html", code);

        let mut description = summary.to_string();
        // 关联个股代码（如 0.301191 / 90.BK0666）附在正文后
        if let Some(stocks) = item["stockList"].as_array() {
            let codes: Vec<&str> = stocks.iter().filter_map(|v| v.as_str()).collect();
            if !codes.is_empty() {
                description.push_str(&format!("<br>关联个股/板块: {}", codes.join(", ")));
            }
        }
        if description.is_empty() {
            description = title.to_string();
        }

        let pub_date = if show_time.is_empty() {
            now()
        } else {
            datetime_str_to_rss(show_time).unwrap_or_else(now)
        };

        let rss_item = ItemBuilder::default()
            .title(Some(title.to_string()))
            .link(link)
            .pub_date(pub_date)
            .description(Some(description))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("东方财富快讯 - {}", column_name))
        .link("https://kuaixun.eastmoney.com/7_24.html")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
