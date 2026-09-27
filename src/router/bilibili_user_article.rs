use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

///把 B 站富文本片段洗成干净 RSS HTML：纯文本段落 + 图片（补全 https、去掉组件标签和脚本钩子）
fn clean_rich_html(fragment: &str) -> String {
    let doc = Html::parse_fragment(fragment);
    let text: String = doc
        .root_element()
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        // 剔除 B 站图片查看器控件文案
        .replace("收起 查看大图 向左旋转 向右旋转", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let mut out = String::new();
    if !text.is_empty() {
        // 转义 & < >，避免破坏 XML
        let escaped = text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        out.push_str(&format!("<p>{}</p>", escaped));
    }
    let image_sel = Selector::parse("img").unwrap();
    for img in doc.select(&image_sel) {
        if let Some(src) = img.value().attr("src") {
            let url = if let Some(rest) = src.strip_prefix("//") {
                format!("https://{}", rest)
            } else if src.starts_with("http") {
                src.to_string()
            } else {
                format!("https://{}", src)
            };
            // 跳过空地址图片（如 src="//"）
            if url == "https:" || url == "https://" || url == "https:///" {
                continue;
            }
            out.push_str(&format!("<img src=\"{}\">", url));
        }
    }
    out
}

///取富文本的纯文本（压空白），用作标题
fn text_of(fragment: &str) -> String {
    let doc = Html::parse_fragment(fragment);
    doc.root_element()
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let uid = para.get("uid").ok_or_else(|| anyhow!("缺少 uid 参数"))?;

    let url = format!(
        "https://api.bilibili.com/x/polymer/web-dynamic/v1/opus/feed/space?host_mid={}",
        uid
    );
    let referer = format!("https://space.bilibili.com/{}/article", uid);
    let headers: Vec<(&str, &str)> = vec![("Referer", referer.as_str())];

    let json: Value = serde_json::from_str(
        fetch_reqwest_get_with_headers(&url, &headers)
            .await?
            .as_str(),
    )?;

    let data = json
        .pointer("/data/items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("找不到 data/items 字段或不是数组"))?;

    let mut item_vec = Vec::new();
    let mut author = String::from("UP主");

    for item in data {
        let content = item["content"].as_str().unwrap_or_default();
        let jump_url = item["jump_url"].as_str().unwrap_or_default();
        let link = if jump_url.starts_with("http") {
            jump_url.to_string()
        } else {
            format!("https:{}", jump_url)
        };

        if let Some(name) = item.pointer("/author/name").and_then(Value::as_str)
            && author == "UP主"
        {
            author = name.to_string();
        }

        // 获取详情页解析描述
        let detail_html = fetch_reqwest_get_with_headers(
            &link,
            &[("Referer", referer.as_str()), ("User-Agent", UA_CHROME)],
        )
        .await
        .unwrap_or_default();

        /* 优先用详情页正文，取不到则用列表接口的 content；两者都过 clean_rich_html 清洗 */
        let description = if !detail_html.is_empty() {
            let document = Html::parse_document(&detail_html);
            let selector = Selector::parse("div.opus-module-content").unwrap();
            document
                .select(&selector)
                .next()
                .map(|el| clean_rich_html(&el.inner_html()))
                .unwrap_or_else(|| clean_rich_html(content))
        } else {
            clean_rich_html(content)
        };

        /* 标题：取纯文本，截断防过长 */
        let title = text_of(content);
        let title: String = title.chars().take(60).collect();
        let title = if title.is_empty() {
            "（无标题）".to_string()
        } else {
            title
        };

        let pub_date = now();

        let rss_item = ItemBuilder::default()
            .title(Some(title))
            .link(link)
            .description(description)
            .pub_date(pub_date)
            .author(author.clone())
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("{} 的 bilibili 图文", author))
        .link(format!("https://space.bilibili.com/{}/article", uid))
        .description(format!("{} 的 bilibili 图文", author))
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}