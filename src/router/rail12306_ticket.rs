use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use std::collections::HashMap;
use tracing::debug;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

///12306 售票信息：查询两站之间的车次余票。
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let date = para
        .get("date")
        .cloned()
        .ok_or_else(|| anyhow!("缺少 date 参数，格式 YYYY-MM-DD"))?;
    let from = para
        .get("from")
        .cloned()
        .ok_or_else(|| anyhow!("缺少 from 参数，始发站名"))?;
    let to = para
        .get("to")
        .cloned()
        .ok_or_else(|| anyhow!("缺少 to 参数，到达站名"))?;

    let client = client_with_cookie()?;

    // 1. 访问首页建立会话
    client
        .get("https://www.12306.cn/index/index.html")
        .header("User-Agent", UA)
        .send()
        .await
        .map_err(|e| anyhow!("访问首页失败: {}", e))?;

    // 2. 初始化登录页
    client
        .get("https://kyfw.12306.cn/otn/login/init")
        .header("User-Agent", UA)
        .send()
        .await
        .map_err(|e| anyhow!("初始化会话失败: {}", e))?;

    // 3. 解析车站编码
    let station_resp = client
        .get("https://kyfw.12306.cn/otn/resources/js/framework/station_name.js")
        .header("User-Agent", UA)
        .header("Referer", "https://kyfw.12306.cn/otn/leftTicket/init")
        .send()
        .await
        .map_err(|e| anyhow!("获取车站列表失败: {}", e))?
        .text()
        .await
        .map_err(|e| anyhow!("读取车站列表失败: {}", e))?;

    let (from_code, to_code, code_map) = parse_station_codes(&station_resp, &from, &to)?;

    // 4. 打开余票查询页获取 JSESSIONID
    let init_url = format!(
        "https://kyfw.12306.cn/otn/leftTicket/init?linktypeid=dc&fs={}&ts={}&date={}&flag=N,N,Y",
        from_code, to_code, date
    );
    client
        .get(&init_url)
        .header("User-Agent", UA)
        .header("Referer", "https://www.12306.cn/index/index.html")
        .send()
        .await
        .map_err(|e| anyhow!("初始化余票页失败: {}", e))?;

    // 5. 查询余票（纳入限流缓存，同参数 TTL 内不重复请求）
    let query_url = format!(
        "https://kyfw.12306.cn/otn/leftTicket/queryG?leftTicketDTO.train_date={}&leftTicketDTO.from_station={}&leftTicketDTO.to_station={}&purpose_codes=ADULT",
        date, from_code, to_code
    );
    let cache_key = crate::rate_limit::make_key("GET", &query_url, &[]);
    let query_resp = if let Some(ttl) = crate::rate_limit::current_ttl() {
        if let Some(cached) = crate::rate_limit::get_cached(&cache_key, ttl) {
            debug!("cache hit: {}", query_url);
            cached
        } else {
            let body = client
                .get(&query_url)
                .header("User-Agent", UA)
                .header("Referer", &init_url)
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| anyhow!("查询余票失败: {}", e))?
                .text()
                .await
                .map_err(|e| anyhow!("读取余票失败: {}", e))?;
            crate::rate_limit::store(&cache_key, &body);
            body
        }
    } else {
        client
            .get(&query_url)
            .header("User-Agent", UA)
            .header("Referer", &init_url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| anyhow!("查询余票失败: {}", e))?
            .text()
            .await
            .map_err(|e| anyhow!("读取余票失败: {}", e))?
    };

    let json: serde_json::Value = serde_json::from_str(&query_resp)?;
    let result = json["data"]["result"]
        .as_array()
        .ok_or_else(|| anyhow!("没有找到相关车次，请检查参数是否正确"))?;

    let mut item_vec = Vec::new();
    for row in result {
        let raw = row.as_str().unwrap_or("");
        let decoded = decode_row(raw);
        let f: Vec<&str> = decoded.split('|').collect();
        if f.len() < 12 {
            continue;
        }
        let train_no = f[3];
        let from_code = f[6];
        let to_code = f[7];
        let from_name = code_map.get(from_code).cloned().unwrap_or_else(|| from_code.to_string());
        let to_name = code_map.get(to_code).cloned().unwrap_or_else(|| to_code.to_string());
        let start_time = f[8];
        let arrive_time = f[9];
        let duration = f[10];

        let title = format!(
            "{} → {} {} {} {}",
            from_name, to_name, start_time, arrive_time, duration
        );
        let mut desc = String::new();
        desc.push_str(&format!("车次：{}<br>", train_no));
        desc.push_str(&format!(
            "始发站：{} → {}<br>",
            from_name, to_name
        ));
        desc.push_str(&format!("出发时间：{}<br>", start_time));
        desc.push_str(&format!("到达时间：{}<br>", arrive_time));
        desc.push_str(&format!("历时：{}<br>", duration));
        desc.push_str(&format!("商务座/特等座：{}<br>", f.get(32).copied().unwrap_or("无")));
        desc.push_str(&format!("一等座：{}<br>", f.get(31).copied().unwrap_or("无")));
        desc.push_str(&format!("二等座/二等包座：{}<br>", f.get(30).copied().unwrap_or("无")));
        desc.push_str(&format!("高级软卧：{}<br>", f.get(29).copied().unwrap_or("无")));
        desc.push_str(&format!("软卧/一等卧：{}<br>", f.get(28).copied().unwrap_or("无")));
        desc.push_str(&format!("动卧：{}<br>", f.get(27).copied().unwrap_or("无")));
        desc.push_str(&format!("硬卧/二等卧：{}<br>", f.get(26).copied().unwrap_or("无")));
        desc.push_str(&format!("软座：{}<br>", f.get(25).copied().unwrap_or("无")));
        desc.push_str(&format!("硬座：{}<br>", f.get(24).copied().unwrap_or("无")));
        desc.push_str(&format!("无座：{}<br>", f.get(23).copied().unwrap_or("无")));
        desc.push_str(&format!("其他：{}", f.get(22).copied().unwrap_or("无")));

        let rss_item = ItemBuilder::default()
            .title(Some(title))
            .link(init_url.clone())
            .description(Some(desc))
            .guid(Some(rss::Guid {
                value: format!(
                    "{}|{}|{}|{}|{}|{}",
                    train_no, from_name, to_name, start_time, arrive_time, duration
                ),
                permalink: false,
            }))
            .build();
        item_vec.push(rss_item);
    }

    let channel = ChannelBuilder::default()
        .title(format!("{} → {} {}", from, to, date))
        .link(init_url)
        .description("12306 售票信息")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

///从 station_name.js 中解析车站编码及 code→name 映射
fn parse_station_codes(
    data: &str,
    from: &str,
    to: &str,
) -> Result<(String, String, HashMap<String, String>)> {
    let mut from_code = String::new();
    let mut to_code = String::new();
    let mut code_map: HashMap<String, String> = HashMap::new();
    for part in data.split('@') {
        let f: Vec<&str> = part.split('|').collect();
        if f.len() >= 4 {
            code_map.insert(f[2].to_string(), f[1].to_string());
            if f[1] == from {
                from_code = f[2].to_string();
            }
            if f[1] == to {
                to_code = f[2].to_string();
            }
        }
    }
    if from_code.is_empty() || to_code.is_empty() {
        return Err(anyhow!("无法识别车站名，请检查 from/to 参数"));
    }
    Ok((from_code, to_code, code_map))
}

///解码 queryG 返回的 result 行：先 URL 解码，再 split('|')
fn decode_row(raw: &str) -> String {
    use urlencoding::decode as urldecode;
    urldecode(raw).map(|c| c.into_owned()).unwrap_or_else(|_| raw.to_string())
}
