use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use md5::{Digest, Md5};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/* B站自 2023 年起对部分 Web 接口启用 WBI 签名鉴权：
旧版 /x/v2/reply 已对所有视频一律返回空数据，评论须改走 /x/v2/reply/wbi/main，
请求参数需附加 wts（秒级时间戳）与 w_rid（md5 签名）。
算法与混淆表出自 bilibili-API-collect：docs/misc/sign/wbi.md */
const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

const WBI_HEADERS: &[(&str, &str)] = &[
    ("User-Agent", UA_CHROME),
    ("Referer", "https://www.bilibili.com/"),
];

/* img_key + sub_key 拼接后按 MIXIN_KEY_ENC_TAB 重排，取前 32 位。
口令每日更替，通过 rate_limit 缓存 1 小时，减少一次 nav 请求 */
async fn wbi_mixin_key() -> Result<String, Error> {
    const CACHE_KEY: &str = "wbi|mixin_key|bilibili.com";
    if let Some(cached) =
        crate::rate_limit::get_cached(CACHE_KEY, std::time::Duration::from_secs(3600))
    {
        return Ok(cached);
    }

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers("https://api.bilibili.com/x/web-interface/nav", WBI_HEADERS)
            .await?
            .as_str(),
    )?;

    /* img_url / sub_url 形如 https://i0.hdslb.com/bfs/wbi/<key>.png，
    <key> 即口令本体，切勿真的去访问这两个 url */
    let take_key = |url: Option<&str>| -> Option<String> {
        let name = url?.rsplit_once('/')?.1;
        name.split_once('.').map(|(k, _)| k.to_string())
    };
    let img = take_key(
        json.pointer("/data/wbi_img/img_url")
            .and_then(Value::as_str),
    )
    .ok_or_else(|| anyhow!("nav 接口未返回 wbi img_key"))?;
    let sub = take_key(
        json.pointer("/data/wbi_img/sub_url")
            .and_then(Value::as_str),
    )
    .ok_or_else(|| anyhow!("nav 接口未返回 wbi sub_key"))?;

    let raw = format!("{}{}", img, sub);
    let mixin: String = MIXIN_KEY_ENC_TAB
        .iter()
        .take(32)
        .map(|&i| raw.as_bytes()[i] as char)
        .collect();

    crate::rate_limit::store(CACHE_KEY, &mixin);
    Ok(mixin)
}

/* 与 JS 的 encodeURIComponent 行为对齐：!'()* 一律剔除（按 bilibili-API-collect 约定），
空格编码为 %20，百分号字母大写 */
fn url_encode(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
                Some(c.to_string())
            } else if "!'()*".contains(c) {
                None
            } else {
                let mut buf = [0u8; 4];
                Some(
                    c.encode_utf8(&mut buf)
                        .bytes()
                        .map(|b| format!("%{:02X}", b))
                        .collect(),
                )
            }
        })
        .collect()
}

/* 给参数列表附加 wts + w_rid，返回可直接拼到 url 后的 query */
fn wbi_sign_query_with_wts(
    mut params: Vec<(&'static str, String)>,
    mixin_key: &str,
    wts: u64,
) -> String {
    params.push(("wts", wts.to_string()));
    params.sort_by(|a, b| a.0.cmp(b.0));
    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    let mut md5 = Md5::new();
    md5.update(query.as_bytes());
    md5.update(mixin_key.as_bytes());
    format!("{}&w_rid={}", query, hex::encode(md5.finalize()))
}

fn wbi_sign_query(params: Vec<(&'static str, String)>, mixin_key: &str) -> String {
    let wts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    wbi_sign_query_with_wts(params, mixin_key, wts)
}

async fn get_aid_from_bvid(bvid: &str) -> Result<i64, Error> {
    let url = format!(
        "https://api.bilibili.com/x/web-interface/view?bvid={}",
        bvid
    );
    let json: Value = serde_json::from_str(fetch_reqwest_get(&url).await?.as_str())?;
    json.pointer("/data/aid")
        .and_then(Value::as_i64)
        .ok_or_else(|| anyhow!("无法从 BVID 获取 AID"))
}

async fn get_video_name(bvid: &str, aid: Option<i64>) -> String {
    let url = if let Some(a) = aid {
        format!("https://api.bilibili.com/x/web-interface/view?aid={}", a)
    } else {
        format!(
            "https://api.bilibili.com/x/web-interface/view?bvid={}",
            bvid
        )
    };
    fetch_reqwest_get(&url)
        .await
        .ok()
        .and_then(|s| {
            serde_json::from_str::<Value>(&s).ok().and_then(|v| {
                v.pointer("/data/title")
                    .and_then(Value::as_str)
                    .map(|t| t.to_string())
            })
        })
        .unwrap_or_else(|| bvid.to_string())
}

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let bvid = para.get("bvid").ok_or_else(|| anyhow!("缺少 bvid 参数"))?;

    let aid = get_aid_from_bvid(bvid).await?;
    let name = get_video_name(bvid, Some(aid)).await;

    let link = format!("https://www.bilibili.com/video/{}", bvid);
    let mixin_key = wbi_mixin_key().await?;
    let query = wbi_sign_query(
        vec![
            ("oid", aid.to_string()),
            ("type", "1".to_string()),
            ("sort", "0".to_string()),
            ("pn", "1".to_string()),
            ("ps", "20".to_string()),
            ("plat", "1".to_string()),
            ("web_location", "1315875292".to_string()),
        ],
        &mixin_key,
    );
    let url = format!("https://api.bilibili.com/x/v2/reply/wbi/main?{}", query);

    let cookie = load_cookie_header(Some("bilibili.com"))
        .ok()
        .flatten()
        .unwrap_or_default();

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(
            &url,
            &[
                ("User-Agent", UA_CHROME),
                ("Referer", link.as_str()),
                ("Cookie", cookie.as_str()),
            ],
        )
        .await?
        .as_str(),
    )?;

    let replies = json
        .pointer("/data/replies")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut item_vec = Vec::new();

    for reply in replies {
        let uname = reply
            .pointer("/member/uname")
            .and_then(Value::as_str)
            .unwrap_or("未知用户");
        let message = reply
            .pointer("/content/message")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let ctime = reply["ctime"].as_i64().unwrap_or(0);
        let rpid = reply["rpid"].as_i64().unwrap_or(0);

        let title = format!("{} : {}", uname, message);
        let description = title.clone();
        let pub_date = timestamp_to_rss(ctime);
        let reply_link = format!("{}/#reply{}", link, rpid);

        let item = ItemBuilder::default()
            .title(Some(title))
            .link(reply_link)
            .description(description)
            .pub_date(pub_date)
            .build();
        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("{} 的 评论", name))
        .link(link)
        .description(format!("{} 的评论", name))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

/* 签名实现用 bilibili-API-collect 文档中的官方测试向量锚定 */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mixin_key() {
        let img = "7cd084941338484aae1ad9425b84077c";
        let sub = "4932caff0ff746eab6f01bf08b70ac45";
        let raw = format!("{}{}", img, sub);
        let mixin: String = MIXIN_KEY_ENC_TAB
            .iter()
            .take(32)
            .map(|&i| raw.as_bytes()[i] as char)
            .collect();
        assert_eq!(mixin, "ea1db124af3c7062474693fa704f4ff8");
    }

    #[test]
    fn test_wbi_sign_query() {
        let params = vec![
            ("foo", "114".to_string()),
            ("bar", "514".to_string()),
            ("zab", "1919810".to_string()),
        ];
        let mixin_key = "ea1db124af3c7062474693fa704f4ff8";
        assert_eq!(
            wbi_sign_query_with_wts(params, mixin_key, 1702204169),
            "bar=514&foo=114&wts=1702204169&zab=1919810&w_rid=8f6f2b5b3d485fe1886cec6a0be8c5d4"
        );
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("五一四"), "%E4%BA%94%E4%B8%80%E5%9B%9B");
        assert_eq!(url_encode("one one four"), "one%20one%20four");
        assert_eq!(url_encode("a!'()*b"), "ab");
        assert_eq!(url_encode("a-_.~b"), "a-_.~b");
    }
}
