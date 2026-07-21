use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

struct UrlResult {
    url: String,
    text: String,
}

fn get_title(data: &Value) -> String {
    let major = match data.pointer("/module_dynamic/major") {
        Some(v) if !v.is_null() => v,
        _ => return String::new(),
    };

    if let Some(tips) = major.pointer("/none/tips").and_then(Value::as_str) {
        return tips.to_string();
    }
    if let Some(courses) = major.pointer("/courses") {
        if !courses.is_null() {
            let title = courses
                .pointer("/title")
                .and_then(Value::as_str)
                .unwrap_or("");
            let sub_title = courses
                .pointer("/sub_title")
                .and_then(Value::as_str)
                .unwrap_or("");
            return format!("{} - {}", title, sub_title).trim().to_string();
        }
    }
    if let Some(content) = major.pointer("/live_rcmd/content").and_then(Value::as_str) {
        if let Ok(card) = serde_json::from_str::<Value>(content) {
            if let Some(title) = card
                .pointer("/live_play_info/title")
                .and_then(Value::as_str)
            {
                return title.to_string();
            }
        }
    }
    let typ = major
        .pointer("/type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .replace("MAJOR_TYPE_", "")
        .to_lowercase();
    if typ.is_empty() {
        return String::new();
    }
    major
        .pointer(format!("/{}/title", typ).as_str())
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn get_description(data: &Value) -> String {
    let mut desc = String::new();
    if let Some(text) = data
        .pointer("/module_dynamic/desc/text")
        .and_then(Value::as_str)
    {
        desc.push_str(text);
    }

    let major = match data.pointer("/module_dynamic/major") {
        Some(v) if !v.is_null() => v,
        _ => return desc,
    };

    if let Some(common_desc) = major.pointer("/common/desc").and_then(Value::as_str) {
        if !desc.is_empty() {
            desc.push_str(&format!("<br>//转发自: {}", common_desc));
        } else {
            desc.push_str(common_desc);
        }
        return desc;
    }

    if major.pointer("/live").is_some() {
        let first = major
            .pointer("/live/desc_first")
            .and_then(Value::as_str)
            .unwrap_or("");
        let second = major
            .pointer("/live/desc_second")
            .and_then(Value::as_str)
            .unwrap_or("");
        return format!("{}<br>{}", first, second);
    }

    if let Some(content) = major.pointer("/live_rcmd/content").and_then(Value::as_str) {
        if let Ok(card) = serde_json::from_str::<Value>(content) {
            let live_play_info = card.pointer("/live_play_info");
            let area_name = live_play_info
                .and_then(|v| v.pointer("/area_name"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let watched = live_play_info
                .and_then(|v| v.pointer("/watched_show/text_large"))
                .and_then(Value::as_str)
                .unwrap_or("");
            return format!("{}·{}", area_name, watched);
        }
    }

    if let Some(summary) = major.pointer("/opus/summary/text").and_then(Value::as_str) {
        return summary.to_string();
    }

    let typ = major
        .pointer("/type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .replace("MAJOR_TYPE_", "")
        .to_lowercase();
    if !typ.is_empty() {
        if let Some(t) = major
            .pointer(format!("/{}/desc", typ).as_str())
            .and_then(Value::as_str)
        {
            return t.to_string();
        }
    }

    desc
}

fn get_iframe(data: &Value, embed: bool) -> String {
    if !embed {
        return String::new();
    }
    let aid = data
        .pointer("/module_dynamic/major/archive/aid")
        .and_then(Value::as_u64)
        .map(|n| n.to_string());
    let bvid = data
        .pointer("/module_dynamic/major/archive/bvid")
        .and_then(Value::as_str)
        .map(String::from);
    if aid.is_none() && bvid.is_none() {
        return String::new();
    }
    format!(
        r#"<iframe width="640" height="360" src="https://www.bilibili.com/blackboard/html5mobileplayer.html?aid={}&amp;cid=undefined&amp;bvid={}" frameborder="0" allowfullscreen="" referrerpolicy="no-referrer"></iframe>"#,
        aid.as_deref().unwrap_or(""),
        bvid.as_deref().unwrap_or("")
    )
}

fn get_imgs(data: &Value) -> String {
    let mut img_urls = Vec::new();
    let major = match data.pointer("/module_dynamic/major") {
        Some(v) if !v.is_null() => v,
        _ => return String::new(),
    };

    if let Some(pics) = major.pointer("/opus/pics").and_then(Value::as_array) {
        for pic in pics {
            if let Some(url) = pic.pointer("/url").and_then(Value::as_str) {
                img_urls.push(format!("<img src=\"{}\">", url));
            }
        }
    }
    if let Some(covers) = major.pointer("/article/covers").and_then(Value::as_array) {
        for cover in covers {
            if let Some(url) = cover.as_str() {
                img_urls.push(format!("<img src=\"{}\">", url));
            }
        }
    }
    if let Some(items) = major.pointer("/draw/items").and_then(Value::as_array) {
        for item in items {
            if let Some(url) = item.pointer("/src").and_then(Value::as_str) {
                img_urls.push(format!("<img src=\"{}\">", url));
            }
        }
    }
    if let Some(content) = major.pointer("/live_rcmd/content").and_then(Value::as_str) {
        if let Ok(card) = serde_json::from_str::<Value>(content) {
            if let Some(url) = card
                .pointer("/live_play_info/cover")
                .and_then(Value::as_str)
            {
                img_urls.push(format!("<img src=\"{}\">", url));
            }
        }
    }
    let typ = major
        .pointer("/type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .replace("MAJOR_TYPE_", "")
        .to_lowercase();
    if let Some(cover) = major
        .pointer(format!("/{}/cover", typ).as_str())
        .and_then(Value::as_str)
    {
        img_urls.push(format!("<img src=\"{}\">", cover));
    }

    img_urls.join("<br>")
}

fn get_url(item: &Value, use_avid: bool) -> Option<UrlResult> {
    let data = item.pointer("/modules").unwrap_or(&Value::Null);
    let major = data
        .pointer("/module_dynamic/major")
        .unwrap_or(&Value::Null);
    if major.is_null() {
        return None;
    }
    let typ = major.pointer("/type").and_then(Value::as_str).unwrap_or("");

    let (url, text) = match typ {
        "MAJOR_TYPE_UGC_SEASON" => {
            let url = major
                .pointer("/ugc_season/jump_url")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            (
                url.clone(),
                format!("合集地址：<a href={}> {}</a>", url, url),
            )
        }
        "MAJOR_TYPE_ARTICLE" => {
            let id = major
                .pointer("/article/id")
                .and_then(Value::as_u64)
                .map(|n| n.to_string())
                .or_else(|| {
                    major
                        .pointer("/article/id")
                        .and_then(Value::as_str)
                        .map(String::from)
                })
                .unwrap_or_default();
            let url = format!("https://www.bilibili.com/read/cv{}", id);
            (
                url.clone(),
                format!("专栏地址：<a href={}>{}</a>", url, url),
            )
        }
        "MAJOR_TYPE_ARCHIVE" => {
            let archive = major.pointer("/archive").unwrap_or(&Value::Null);
            let bvid = archive
                .pointer("/bvid")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let aid = archive
                .pointer("/aid")
                .and_then(Value::as_u64)
                .map(|n| n.to_string());
            let id = if use_avid {
                aid.as_ref()
                    .map(|s| format!("av{}", s))
                    .unwrap_or_else(|| bvid.clone())
            } else {
                bvid.clone()
            };
            let url = format!("https://www.bilibili.com/video/{}", id);
            (
                url.clone(),
                format!("视频地址：<a href={}>{}</a>", url, url),
            )
        }
        "MAJOR_TYPE_COMMON" => {
            let url = major
                .pointer("/common/jump_url")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            (url.clone(), format!("地址：<a href={}>{}</a>", url, url))
        }
        "MAJOR_TYPE_OPUS" => {
            let item_type = item.pointer("/type").and_then(Value::as_str).unwrap_or("");
            let jump_url = major
                .pointer("/opus/jump_url")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let url = if jump_url.starts_with("http") {
                jump_url.clone()
            } else {
                format!("https:{}", jump_url)
            };
            let text = if item_type == "DYNAMIC_TYPE_ARTICLE" {
                format!("专栏地址：<a href={}>{}</a>", url, url)
            } else {
                format!("图文地址：<a href={}>{}</a>", url, url)
            };
            (url, text)
        }
        "MAJOR_TYPE_PGC" => {
            let pgc = major.pointer("/pgc").unwrap_or(&Value::Null);
            let url = format!(
                "https://www.bilibili.com/bangumi/play/ep{}&season_id={}",
                pgc.pointer("/epid")
                    .and_then(Value::as_u64)
                    .map(|n| n.to_string())
                    .unwrap_or_default(),
                pgc.pointer("/season_id")
                    .and_then(Value::as_u64)
                    .map(|n| n.to_string())
                    .unwrap_or_default()
            );
            (
                url.clone(),
                format!("剧集地址：<a href={}>{}</a>", url, url),
            )
        }
        "MAJOR_TYPE_COURSES" => {
            let id = major
                .pointer("/courses/id")
                .and_then(Value::as_u64)
                .map(|n| n.to_string())
                .unwrap_or_default();
            let url = format!("https://www.bilibili.com/cheese/play/ss{}", id);
            (
                url.clone(),
                format!("课程地址：<a href={}>{}</a>", url, url),
            )
        }
        "MAJOR_TYPE_MUSIC" => {
            let id = major
                .pointer("/music/id")
                .and_then(Value::as_u64)
                .map(|n| n.to_string())
                .unwrap_or_default();
            let url = format!("https://www.bilibili.com/audio/au{}", id);
            (
                url.clone(),
                format!("音频地址：<a href={}>{}</a>", url, url),
            )
        }
        "MAJOR_TYPE_LIVE" => {
            let id = major
                .pointer("/live/id")
                .and_then(Value::as_u64)
                .map(|n| n.to_string())
                .unwrap_or_default();
            let url = format!("https://live.bilibili.com/{}", id);
            (
                url.clone(),
                format!("直播间地址：<a href={}>{}</a>", url, url),
            )
        }
        "MAJOR_TYPE_LIVE_RCMD" => {
            let mut live_play_info = None;
            if let Some(content) = major.pointer("/live_rcmd/content").and_then(Value::as_str) {
                if let Ok(card) = serde_json::from_str::<Value>(content) {
                    live_play_info = card.pointer("/live_play_info").cloned();
                }
            }
            let room_id = live_play_info
                .and_then(|v: Value| v.pointer("/room_id").cloned())
                .and_then(|arg0: Value| Value::as_u64(&arg0))
                .map(|n| n.to_string())
                .unwrap_or_default();
            let url = format!("https://live.bilibili.com/{}", room_id);
            (
                url.clone(),
                format!("直播间地址：<a href={}>{}</a>", url, url),
            )
        }
        _ => return None,
    };
    Some(UrlResult { url, text })
}

fn build_cookie_header() -> Result<Option<String>> {
    load_cookie_header(Some("bilibili.com"))
}

pub fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let uid = para
        .get("uid")
        .ok_or_else(|| anyhow!("缺少 uid 参数，请在 /bilibili_dynamic?uid=xxx 中传入 uid"))?;

    let show_emoji = parse_bool(para.get("showEmoji"), false);
    let embed = parse_bool(para.get("embed"), true);
    let use_avid = parse_bool(para.get("useAvid"), false);
    let direct_link = parse_bool(para.get("directLink"), false);
    let hide_goods = parse_bool(para.get("hideGoods"), false);
    let offset = para.get("offset").cloned().unwrap_or_default();

    let api_url = if offset.is_empty() {
        format!(
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/space?host_mid={}&platform=web&features=itemOpusStyle,listOnlyfans,opusBigCover,onlyfansVote",
            uid
        )
    } else {
        format!(
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/space?host_mid={}&platform=web&offset={}&features=itemOpusStyle,listOnlyfans,opusBigCover,onlyfansVote",
            uid,
            urlencoding::encode(&offset)
        )
    };

    let cookie_header = build_cookie_header().ok().flatten();
    let referer = format!("https://space.bilibili.com/{}/", uid);
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    let mut headers: Vec<(&str, &str)> = vec![("Referer", &referer), ("User-Agent", ua)];
    if let Some(cookie) = &cookie_header {
        headers.push(("Cookie", cookie));
    }

    let response = fetch_reqwest_get_with_headers(&api_url, &headers)?;
    let json: Value = match serde_json::from_str(&response) {
        Ok(v) => v,
        Err(e) => {
            if response.contains("412") || response.contains("security") {
                return Err(anyhow!("B站API需要登录Cookie才能访问，请配置 cookies.json (bilibili.com)"));
            }
            return Err(anyhow!("JSON解析失败: {} — 响应片段: {}", e, &response.chars().take(200).collect::<String>()));
        }
    };
    if json.pointer("/code").and_then(Value::as_i64) != Some(0) {
        return Err(anyhow!("B站API返回错误 code={}: {}", json.pointer("/code").and_then(Value::as_i64).unwrap_or(-1), json.pointer("/message").and_then(Value::as_str).unwrap_or("")));
    }

    let items = json
        .pointer("/data/items")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("找不到 data.items 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    let mut author = String::new();

    for item in items {
        if hide_goods
            && item
                .pointer("/modules/module_dynamic/additional/type")
                .and_then(Value::as_str)
                == Some("ADDITIONAL_TYPE_GOODS")
        {
            continue;
        }

        let modules = item.pointer("/modules").unwrap_or(&Value::Null);
        if let Some(name) = modules
            .pointer("/module_author/name")
            .and_then(Value::as_str)
        {
            if author.is_empty() {
                author = name.to_string();
            }
        }

        let mut description = get_description(modules);
        let original_title = get_title(modules);

        if show_emoji {
            if let Some(nodes) = modules
                .pointer("/module_dynamic/desc/rich_text_nodes")
                .and_then(Value::as_array)
            {
                for node in nodes {
                    if let Some(emoji) = node.pointer("/emoji") {
                        if let (Some(text), Some(icon_url)) = (
                            node.pointer("/text").and_then(Value::as_str),
                            emoji.pointer("/icon_url").and_then(Value::as_str),
                        ) {
                            description = description.replace(
                                text,
                                &format!(
                                    r#"<img alt="{}" src="{}" style="margin: -1px 1px 0px; display: inline-block; width: 20px; height: 20px; vertical-align: text-bottom;" referrerpolicy="no-referrer">"#,
                                    text, icon_url
                                ),
                            );
                        }
                    }
                    if let Some(pics) = node.pointer("/pics").and_then(Value::as_array) {
                        if let Some(text) = node.pointer("/text").and_then(Value::as_str) {
                            let replacements: Vec<String> = pics
                                .iter()
                                .filter_map(|pic| pic.pointer("/src").and_then(Value::as_str))
                                .map(|src| format!(r#"<img alt="{}" src="{}" style="display:inline-block; max-width:100%;" referrerpolicy="no-referrer">"#, text, src))
                                .collect();
                            if !replacements.is_empty() {
                                description = description.replace(text, &replacements.join("<br>"));
                            }
                        }
                    }
                }
            }
        }

        let mut link = item
            .pointer("/id_str")
            .and_then(Value::as_str)
            .map(|id_str| format!("https://t.bilibili.com/{}", id_str))
            .unwrap_or_default();

        let url_result = get_url(item, use_avid);
        let mut url_text = None;
        if let Some(url_result) = &url_result {
            url_text = Some(url_result.text.clone());
            if direct_link {
                link = url_result.url.clone();
            }
        }

        let origin = item.pointer("/orig/modules");
        let origin_url_result = origin.and_then(|origin| get_url(origin, use_avid));
        if let Some(origin_url_result) = &origin_url_result {
            if direct_link {
                link = origin_url_result.url.clone();
            }
        }

        let title = if original_title.is_empty() {
            link.clone()
        } else {
            original_title.clone()
        };

        let mut origin_description = origin
            .and_then(|o| o.pointer("/module_author/name").and_then(Value::as_str))
            .map(|n| format!("//转发自: @{}: ", n))
            .unwrap_or_default();
        if let Some(origin_title) = origin.map(get_title).filter(|s| !s.is_empty()) {
            origin_description.push_str(&origin_title);
        }
        let origin_des = origin.map(get_description).unwrap_or_default();
        if !origin_des.is_empty() {
            if !origin_description.is_empty() {
                origin_description.push_str("<br>");
            }
            origin_description.push_str(&origin_des);
        }

        description = description.replace("\r\n", "<br>").replace('\n', "<br>");
        let origin_description = origin_description
            .replace("\r\n", "<br>")
            .replace('\n', "<br>");

        let mut desc_fields = vec![
            original_title,
            description,
            get_iframe(modules, embed),
            get_imgs(modules),
            url_text.unwrap_or_default(),
            origin_description,
        ];
        if let Some(origin) = origin {
            desc_fields.push(get_iframe(origin, embed));
            desc_fields.push(get_imgs(origin));
            if let Some(origin_url_result) = &origin_url_result {
                desc_fields.push(origin_url_result.text.clone());
            }
        }
        let description = desc_fields
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
            .join("<br>");

        let pub_ts = modules.pointer("/module_author/pub_ts");
        let pub_date = pub_ts
            .and_then(Value::as_i64)
            .or_else(|| {
                pub_ts
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse::<i64>().ok())
            })
            .map(timestamp_to_rss)
            .unwrap_or_else(now);

        let item = ItemBuilder::default()
            .title(Some(no_double_quotes(title)))
            .link(Some(no_double_quotes(link)))
            .description(description)
            .pub_date(Some(pub_date))
            .author(Some(no_double_quotes(
                modules
                    .pointer("/module_author/name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            )))
            .build();

        item_vec.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("{} 的 bilibili 动态", author))
        .link(format!("https://space.bilibili.com/{}/dynamic", uid))
        .description(format!("{} 的 bilibili 动态", author))
        .items(item_vec)
        .build();

    Ok(channel.to_string())
}
