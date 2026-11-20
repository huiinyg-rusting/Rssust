use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let product = para
        .get("product")
        .ok_or_else(|| anyhow!("缺少 product 参数 (live/vc/wh)"))?;

    let product_title = match product.as_str() {
        "live" => "直播",
        "vc" => "小视频",
        "wh" => "相簿",
        _ => return Err(anyhow!("无效的 product 值: {}，可选: live/vc/wh", product)),
    };

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(
            &format!(
                "https://api.vc.bilibili.com/news/v1/notice/list?platform=pc&product={}&category=all&page_no=1&page_size=20",
                product
            ),
            &[("Referer", "https://link.bilibili.com/p/eden/news")],
        ).await?
        .as_str(),
    )?;

    let data = json
        .pointer("/data/items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data/items 字段或不是数组"))?;

    let mut item_vec = Vec::new();

    for item in data {
        let mark = item["mark"].as_str().unwrap_or_default();
        let cover_url = item["cover_url"].as_str().unwrap_or_default();
        let ctime = item["ctime"].as_str().unwrap_or_default();
        /* announce_link 可能是空串，空串同样走兜底链接 */
        let announce_link = item["announce_link"].as_str().filter(|s| !s.is_empty());
        /* id 曾是数字、现为字符串，两种类型都接住 */
        let id = item["id"]
            .as_i64()
            .or_else(|| item["id"].as_str().and_then(|s| s.parse().ok()))
            .unwrap_or_default();

        let description = format!(
            "{}<br><img src=\"{}\" referrerpolicy=\"no-referrer\">",
            mark, cover_url
        );

        let link = announce_link.map(|s| s.to_string()).unwrap_or_else(|| {
            format!(
                "https://link.bilibili.com/p/eden/news#/newsdetail?id={}",
                id
            )
        });

        /* ctime 形如 "2024-01-01 12:00:00"，转成标准 RFC2822 */
        let pub_date = datetime_str_to_rss(ctime).unwrap_or_else(now);

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(item["title"].to_string())))
            .link(link)
            .description(description)
            .pub_date(pub_date)
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("bilibili {}公告", product_title))
        .link(format!(
            "https://link.bilibili.com/p/eden/news#/?tab={}&tag=all&page_no=1",
            product
        ))
        .description(format!("bilibili {}公告", product_title))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
