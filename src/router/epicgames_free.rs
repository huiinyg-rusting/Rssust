use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use chrono::{DateTime, Utc};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

/// Epic 免费游戏（对应 RSSHub /epicgames/freegames）。
/// 上游：https://store-site-backend-static.ak.epicgames.com/freeGamesPromotions
/// 官方公开 JSON（Epic 商店前端自用），无官方 RSS，该域名无 robots 限制。
/// 只挑“0 折扣”的促销窗口：当前可领的排前面，其次是将要免费的。
/// 参数：
///  - locale：语言/商店区域，默认 zh-CN
///  - country：国家代码，默认 CN
///  - upcoming：是否包含“即将免费”，默认 true
///  - limit：返回条数，默认 20
///
/// 限流：建议配置 routes.rate_limit["/epicgames_free"] = 1800（每周更新一次即可）
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let locale = para
        .get("locale")
        .cloned()
        .unwrap_or_else(|| "zh-CN".to_string());
    let country = para
        .get("country")
        .cloned()
        .unwrap_or_else(|| "CN".to_string());
    let include_upcoming = parse_bool(para.get("upcoming"), true);
    let limit = opt_usize(&para, "limit", 20);

    let url = format!(
        "https://store-site-backend-static.ak.epicgames.com/freeGamesPromotions?locale={}&country={}&allowCountries={}",
        urlencoding::encode(&locale),
        urlencoding::encode(&country),
        urlencoding::encode(&country)
    );
    let resp = fetch_reqwest_get_with_headers(&url, &[("User-Agent", UA_CHROME)]).await?;
    let json: Value =
        serde_json::from_str(&resp).map_err(|e| anyhow!("Epic 返回非 JSON：{}", e))?;
    let elements = json["data"]["Catalog"]["searchStore"]["elements"]
        .as_array()
        .ok_or_else(|| anyhow!("找不到 data.Catalog.searchStore.elements"))?;

    let now_ts = Utc::now();
    let mut current = Vec::new();
    let mut upcoming = Vec::new();

    for game in elements {
        let Some(free) = free_window(game, now_ts) else {
            continue;
        };
        let Some(item) = build_item(game, &free, &locale) else {
            continue;
        };
        if free.started {
            current.push((free.end, item));
        } else if include_upcoming {
            upcoming.push((free.start, item));
        }
    }

    /* 当前可领：按结束时间早的在前；即将免费：按开始时间早的在前 */
    current.sort_by_key(|(key, _)| *key);
    upcoming.sort_by_key(|(key, _)| *key);

    let mut item_vec: Vec<Item> = current.into_iter().map(|(_, item)| item).collect();
    item_vec.extend(upcoming.into_iter().map(|(_, item)| item));
    item_vec.truncate(limit);

    if item_vec.is_empty() {
        return Err(anyhow!("当前没有处于免费/即将免费状态的游戏"));
    }

    let channel = ChannelBuilder::default()
        .title("Epic 免费游戏")
        .link(format!("https://store.epicgames.com/{}/free-games", locale))
        .description("Epic Games Store 限时免费游戏")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

/* 一个 0% 折扣的促销时间窗 */
struct FreeWindow {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    /* 是否已经开始（true=现在可领，false=即将免费） */
    started: bool,
}

/* 在游戏的所有促销窗口里找出“免费（0%）”的那一个：
优先当前正在进行的，其次最近将要开始的。找不到返回 None。 */
fn free_window(game: &Value, now: DateTime<Utc>) -> Option<FreeWindow> {
    let mut best_current: Option<FreeWindow> = None;
    let mut best_upcoming: Option<FreeWindow> = None;

    let groups = [
        game.pointer("/promotions/promotionalOffers"),
        game.pointer("/promotions/upcomingPromotionalOffers"),
    ];
    for group in groups.into_iter().flatten() {
        let Some(wrappers) = group.as_array() else {
            continue;
        };
        for wrapper in wrappers {
            let Some(offers) = wrapper["promotionalOffers"].as_array() else {
                continue;
            };
            for offer in offers {
                if offer["discountSetting"]["discountPercentage"]
                    .as_i64()
                    .unwrap_or(-1)
                    != 0
                {
                    continue;
                }
                let (Some(start), Some(end)) = (
                    parse_time(offer["startDate"].as_str()),
                    parse_time(offer["endDate"].as_str()),
                ) else {
                    continue;
                };
                let started = start <= now && now <= end;
                let window = FreeWindow {
                    start,
                    end,
                    started,
                };
                if started {
                    let earlier_end = best_current.as_ref().is_none_or(|b| end < b.end);
                    if earlier_end {
                        best_current = Some(window);
                    }
                } else if start > now {
                    let sooner = best_upcoming.as_ref().is_none_or(|b| start < b.start);
                    if sooner {
                        best_upcoming = Some(window);
                    }
                }
            }
        }
    }
    best_current.or(best_upcoming)
}

fn parse_time(s: Option<&str>) -> Option<DateTime<Utc>> {
    let s = s?;
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/* 把一条 Epic 促销商品转成 RSS Item（排序键在外面用 FreeWindow 的时间，这里不掺和） */
fn build_item(game: &Value, free: &FreeWindow, locale: &str) -> Option<Item> {
    let title = game["title"].as_str()?.to_string();
    if title.is_empty() {
        return None;
    }
    let id = game["id"].as_str().unwrap_or("");
    let description_text = game["description"].as_str().unwrap_or("").trim();

    let slug = game
        .pointer("/catalogNs/mappings/0/pageSlug")
        .and_then(|v| v.as_str())
        .map(|s| s.trim_end_matches("/home").to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            game["productSlug"]
                .as_str()
                .map(|s| s.trim_end_matches("/home").to_string())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| game["urlSlug"].as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    let link = if slug.is_empty() {
        format!("https://store.epicgames.com/{}/free-games", locale)
    } else {
        format!("https://store.epicgames.com/{}/p/{}", locale, slug)
    };

    let image = game["keyImages"]
        .as_array()
        .and_then(|imgs| {
            imgs.iter()
                .find(|i| i["type"].as_str() == Some("OfferImageWide"))
                .or_else(|| imgs.first())
        })
        .and_then(|i| i["url"].as_str())
        .unwrap_or("");

    let original = game
        .pointer("/price/totalPrice/fmtPrice/originalPrice")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let status = if free.started {
        format!("现在免费，截止 {}", format_time(free.end))
    } else {
        format!(
            "即将免费：{} ~ {}",
            format_time(free.start),
            format_time(free.end)
        )
    };

    let mut description = format!("<img src=\"{}\"><br><b>{}</b>", image, status);
    if !original.is_empty() {
        description.push_str(&format!("<br>原价：{}", original));
    }
    if !description_text.is_empty() {
        description.push_str(&format!("<br><br>{}", description_text));
    }
    description.push_str(&format!("<br><br><a href=\"{}\">前往 Epic 商店</a>", link));

    let item = ItemBuilder::default()
        .title(Some(if free.started {
            title.clone()
        } else {
            format!("[即将免费] {}", title)
        }))
        .link(Some(link))
        .description(Some(description))
        .pub_date(now())
        .guid(rss::Guid {
            value: if id.is_empty() {
                format!("epic-free-{}", title)
            } else {
                id.to_string()
            },
            permalink: false,
        })
        .build();
    Some(item)
}

fn format_time(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap())
        .format("%Y-%m-%d %H:%M")
        .to_string()
}
