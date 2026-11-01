use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

/// Steam 特惠（对应 RSSHub 的 Steam 特惠类路由）。
/// 上游：https://store.steampowered.com/api/featuredcategories 官方公开 JSON，
/// 该接口不在 robots.txt 的 Disallow 列表中，Steam 也没有官方 RSS。
/// 数据来源是首页“特惠”区（specials），一次约 10 条。
/// 参数：
///  - cc：国家/地区代码，默认 cn（价格随之变化）
///  - lang：语言，默认 zh
///  - limit：返回条数，默认 10，最大 20
///
/// 限流：建议配置 routes.rate_limit["/steam_discounts"] = 600
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let cc = para.get("cc").cloned().unwrap_or_else(|| "cn".to_string());
    let lang = para
        .get("lang")
        .cloned()
        .unwrap_or_else(|| "zh".to_string());
    let limit = opt_usize(&para, "limit", 10).min(20);

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

    let items = json["specials"]["items"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 specials.items 字段（该地区可能没有特惠数据）"))?;

    let mut item_vec = Vec::new();
    for game in items {
        /* 只保留真正打折的商品 */
        if !game["discounted"].as_bool().unwrap_or(false) {
            continue;
        }
        let app_id = game["id"].as_i64().unwrap_or(0);
        if app_id == 0 {
            continue;
        }
        let name = game["name"].as_str().unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }
        let percent = game["discount_percent"].as_i64().unwrap_or(0);
        let original = game["original_price"].as_i64().unwrap_or(0);
        let final_price = game["final_price"].as_i64().unwrap_or(0);
        let currency = game["currency"].as_str().unwrap_or("");
        let expiry = game["discount_expiration"].as_i64().unwrap_or(0);
        let image = game["header_image"].as_str().unwrap_or("");

        let currency_symbol = currency_symbol(currency);
        let expiry_text = if expiry > 0 {
            let rss = timestamp_to_rss(expiry);
            format!("<br>折扣截止：{}", rss)
        } else {
            String::new()
        };

        let description = format!(
            "<img src=\"{}\"><br><b>-{}%</b> 原价 {}{:.2} → 现价 <b>{}{:.2}</b>{}<br><a href=\"https://store.steampowered.com/app/{}/\">在 Steam 查看</a>",
            image,
            percent,
            currency_symbol,
            original as f64 / 100.0,
            currency_symbol,
            final_price as f64 / 100.0,
            expiry_text,
            app_id
        );

        item_vec.push(
            ItemBuilder::default()
                .title(Some(format!("{}（-{}%）", name, percent)))
                .link(Some(format!(
                    "https://store.steampowered.com/app/{}/",
                    app_id
                )))
                .description(Some(description))
                .pub_date(now())
                .guid(rss::Guid {
                    value: format!("steam-app-{}", app_id),
                    permalink: false,
                })
                .build(),
        );
        if item_vec.len() >= limit {
            break;
        }
    }

    if item_vec.is_empty() {
        return Err(anyhow!("该地区当前没有抓取到特惠商品"));
    }

    let channel = ChannelBuilder::default()
        .title(format!("Steam 特惠（{}）", cc.to_uppercase()))
        .link("https://store.steampowered.com/specials/")
        .description("Steam 首页特惠区")
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
