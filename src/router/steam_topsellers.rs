use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

/// Steam 全球热销榜（Top Sellers）。
/// 上游：https://store.steampowered.com/api/featuredcategories 的 top_sellers 区，
/// 与 steam_discounts 同接口族，官方公开 JSON；robots.txt 不禁止 /api/，Steam 也没有
/// 热销榜的官方 RSS（仅有游戏新闻 feed）。
/// 参数：
///  - cc：国家/地区代码，默认 cn（价格随地区变化）
///  - lang：语言，默认 schinese
///  - limit：返回条数，默认 20，最大 30（上游一次约 30 条）
///
/// 限流：建议配置 routes.rate_limit["/steam_topsellers"] = 600
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let cc = para.get("cc").cloned().unwrap_or_else(|| "cn".to_string());
    let lang = para
        .get("lang")
        .cloned()
        .unwrap_or_else(|| "schinese".to_string());
    let limit = opt_usize(&para, "limit", 20).min(30);

    let url = format!(
        "https://store.steampowered.com/api/featuredcategories?cc={}&l={}",
        urlencoding::encode(&cc),
        urlencoding::encode(&lang)
    );
    let resp = fetch_reqwest_get_with_headers(
        &url,
        &[("User-Agent", UA_CHROME), ("Accept", "application/json")],
    )
    .await?;
    let json: Value =
        serde_json::from_str(&resp).map_err(|e| anyhow!("Steam 返回非 JSON：{}", e))?;

    let items = json["top_sellers"]["items"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 top_sellers.items 字段（该地区可能没有热销数据）"))?;

    let mut item_vec = Vec::new();
    for game in items {
        let app_id = game["id"].as_i64().unwrap_or(0);
        if app_id == 0 {
            continue;
        }
        let name = game["name"].as_str().unwrap_or("").trim().to_string();
        if name.is_empty() {
            continue;
        }
        let discounted = game["discounted"].as_bool().unwrap_or(false);
        let percent = game["discount_percent"].as_i64().unwrap_or(0);
        let original = game["original_price"].as_i64().unwrap_or(0);
        let final_price = game["final_price"].as_i64().unwrap_or(0);
        let currency = game["currency"].as_str().unwrap_or("");
        let image = game["header_image"].as_str().unwrap_or("");

        let symbol = currency_symbol(currency);
        let title = if discounted {
            format!("{}（-{}%）", name, percent)
        } else {
            name.clone()
        };
        let price_html = if discounted {
            format!(
                "-{}% 原价 {}{:.2} → 现价 <b>{}{:.2}</b>",
                percent,
                symbol,
                original as f64 / 100.0,
                symbol,
                final_price as f64 / 100.0
            )
        } else {
            format!(
                "当前价格 <b>{}{:.2}</b>",
                symbol,
                final_price as f64 / 100.0
            )
        };
        let link = format!("https://store.steampowered.com/app/{}/", app_id);
        let description = format!(
            "<img src=\"{}\"><br>{}<br><a href=\"{}\">在 Steam 查看</a>",
            image, price_html, link
        );

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(Some(link))
                .description(Some(description))
                .pub_date(now())
                .guid(rss::Guid {
                    value: format!("steam-topsellers-{}-{}", cc, app_id),
                    permalink: false,
                })
                .build(),
        );
        if item_vec.len() >= limit {
            break;
        }
    }

    if item_vec.is_empty() {
        return Err(anyhow!("该地区当前没有抓取到热销商品"));
    }

    let channel = ChannelBuilder::default()
        .title(format!("Steam 全球热销榜（{}）", cc.to_uppercase()))
        .link("https://store.steampowered.com/charts/topselling/")
        .description("Steam Top Sellers 全球热销榜")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

/* 货币代码 → 符号（未知货币直接用代码前缀） */
fn currency_symbol(code: &str) -> String {
    match code {
        "CNY" => "¥".to_string(),
        "USD" => "$".to_string(),
        "EUR" => "€".to_string(),
        "GBP" => "£".to_string(),
        "JPY" => "¥".to_string(),
        other => format!("{} ", other),
    }
}
