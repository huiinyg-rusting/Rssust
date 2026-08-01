use anyhow::{Error, Result, anyhow};
use chrono::{DateTime, TimeZone};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};
use serde_json::Value;
use std::fs;
use std::result::Result::Ok;
use std::sync::OnceLock;
use std::{collections::HashMap, env};
use tracing::{debug, warn};

///自定义错误类型，可携带 HTTP 状态码。渲染层会取 `status` 作为状态码、`message` 作为正文。
#[derive(Debug)]
pub struct HttpError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP {}: {}", self.status, self.message)
    }
}

impl std::error::Error for HttpError {}

impl HttpError {
    pub fn new(status: u16, message: impl Into<String>) -> Self {
        HttpError {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(404, message)
    }

    pub fn bad_gateway(message: impl Into<String>) -> Self {
        Self::new(502, message)
    }
}

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build reqwest client")
    })
}

///这个函数序列化从key1=1&key2=2 到{"key1": "2", "key2": "2"}的Hashmap;
pub fn params_to_hashmap(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let value = urlencoding::decode(value)
                .map(|c| c.into_owned())
                .unwrap_or_else(|_| value.to_string());
            params.insert(key.to_string(), value);
        } else if !pair.is_empty() {
            // 处理没有等号的参数（如 "flag"），值设为空字符串
            params.insert(pair.to_string(), String::new());
        }
    }
    params
}

pub fn hashmap_to_params(hashmap: HashMap<String, String>) -> String {
    let mut response: String = "".to_owned();
    for (key, value) in hashmap.iter() {
        response.push_str(format!("{}={}", key, value).as_str());
    }
    response
}

//下面是reqwest get的内容
//不会使用线程池
pub async fn fetch_reqwest_get(url: &str) -> Result<String, Error> {
    debug!("GET {}", url);
    let result = (|| async {
        Ok(client()
            .get(url)
            .send()
            .await
            .map_err(Error::from)?
            .text()
            .await
            .map_err(Error::from)?)
    })()
    .await;
    if let Err(ref e) = result {
        warn!("GET {} failed: {}", url, e);
    }
    result
}

///This can be an array of tuples, or a HashMap, or a custom type that implements Serialize.
///这可以是一个元组数组，或者是一个 HashMap ，或者是一个实现了 Serialize 的自定义类型。
///The feature form is required.
///必须使用 form 功能
pub async fn fetch_reqwest_post(url: &str, body: String) -> Result<String, Error> {
    debug!("POST {}", url);
    let result = (|| async {
        Ok(client()
            .post(url)
            .body(body)
            .send()
            .await
            .map_err(Error::from)?
            .text()
            .await
            .map_err(Error::from)?)
    })()
    .await;
    if let Err(ref e) = result {
        warn!("POST {} failed: {}", url, e);
    }
    result
}

pub async fn fetch_reqwest_post_json(url: &str, json_body: &str) -> Result<String, Error> {
    debug!("POST {} (json)", url);
    let result = (|| async {
        Ok(client()
            .post(url)
            .header("Content-Type", "application/json")
            .body(json_body.to_string())
            .send()
            .await
            .map_err(Error::from)?
            .text()
            .await
            .map_err(Error::from)?)
    })()
    .await;
    if let Err(ref e) = result {
        warn!("POST {} failed: {}", url, e);
    }
    result
}

pub async fn fetch_reqwest_get_with_headers(
    url: &str,
    headers: &[(&str, &str)],
) -> Result<String, Error> {
    debug!("GET {} (with headers)", url);
    let result = (|| async {
        let mut req = client().get(url);
        for &(key, value) in headers {
            req = req.header(key, value);
        }
        Ok(req
            .send()
            .await
            .map_err(Error::from)?
            .text()
            .await
            .map_err(Error::from)?)
    })()
    .await;
    if let Err(ref e) = result {
        warn!("GET {} failed: {}", url, e);
    }
    result
}

///简单的1,true,True转true
pub fn parse_bool(value: Option<&String>, default: bool) -> bool {
    match value.map(String::as_str) {
        Some("1") | Some("true") | Some("True") => true,
        Some("0") | Some("false") | Some("False") => false,
        Some(other) => other.parse().unwrap_or(default),
        None => default,
    }
}

pub fn load_cookie_header(domain_filter: Option<&str>) -> Result<Option<String>> {
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| anyhow!("Could not get executable directory"))?;
    let cookie_path = exe_dir.join("cookies.json");
    let content = fs::read_to_string(cookie_path).map_err(|_| anyhow!("无法读取 cookies.json"))?;
    let cookies: Value = serde_json::from_str(content.as_str())?;
    let cookie_array = cookies
        .as_array()
        .ok_or_else(|| anyhow!("cookies.json 格式错误, 预期数组"))?;
    let cookie_pairs: Vec<String> = cookie_array
        .iter()
        .filter_map(|cookie| {
            if let Some(filter) = domain_filter {
                let domain = cookie.get("domain")?.as_str()?;
                if !domain.contains(filter) {
                    return None;
                }
            }
            let name = cookie.get("name")?.as_str()?;
            let value = cookie.get("value")?.as_str()?;
            Some(format!("{}={}", name, value))
        })
        .collect();
    if cookie_pairs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(cookie_pairs.join("; ")))
    }
}

pub fn now() -> String {
    Local::now().format("%a, %d %b %Y %H:%M:%S %z").to_string()
}

///x月y日到rss用的时间
//这个函数已经测试过有效了
///时间如果无效返回None
pub fn chinese_date_to_parse(input: &str) -> Option<String> {
    let re = regex::Regex::new(r"(\d{1,2})月(\d{1,2})日").ok()?;
    let caps = re.captures(input)?;
    let month = caps.get(1)?.as_str().parse::<u32>().ok()?;
    let day = caps.get(2)?.as_str().parse::<u32>().ok()?;
    let year = Local::now().year() as i32;
    Some(
        NaiveDate::from_ymd_opt(year, month, day)?
            .format("%a, %d %b %Y 00:00:00 +0800")
            .to_string(),
    )
}
///去除首尾双引号
//注意'"'是一对单引号包双引号
pub fn no_double_quotes(s: String) -> String {
    s.trim_matches('"').to_string()
}

//查找环境变量
pub fn env_search(s: &str) -> Option<String> {
    match env::var(s) {
        Ok(i) => Some(i),
        Err(_) => None,
    }
}

//Unix时间戳改RSS标准时间
pub fn timestamp_to_rss(ts: i64) -> String {
    DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
        .unwrap_or_else(now)
}

/// "YYYY-MM-DD HH:MM:SS" 格式的字符串转 RSS pubDate（输入视为东八区本地时间）
pub fn datetime_str_to_rss(datetime_str: &str) -> Option<String> {
    let naive = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S").ok()?;
    let fixed = chrono::FixedOffset::east_opt(8 * 3600)?;
    Some(
        fixed
            .from_local_datetime(&naive)
            .single()?
            .format("%a, %d %b %Y %H:%M:%S %z")
            .to_string(),
    )
}

/// "YYYY-MM-DD HH:MM:SS" 格式的字符串转 RSS pubDate（输入视为 UTC 时间，自动换算为东八区）
pub fn utc_str_to_rss(datetime_str: &str) -> Option<String> {
    let naive = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S").ok()?;
    let fixed = chrono::FixedOffset::east_opt(8 * 3600)?;
    let wall = fixed.from_utc_datetime(&naive).naive_local();
    datetime_str_to_rss(&wall.format("%Y-%m-%d %H:%M:%S").to_string())
}

/// 带浏览器 TLS 指纹伪装的 GET 请求（绕过 Cloudflare 等反爬）
/// 使用 curl-impersonate-cli (curl-impersonate) 模拟 Chrome 124 指纹
/// 需要启用 cargo feature "download" 自动下载预编译二进制
pub async fn fetch_browser_get(url: &str) -> Result<String, Error> {
    debug!("GET (browser) {}", url);
    let opts = curl_impersonate_cli::download::DownloadOptions::default();
    let bin = curl_impersonate_cli::download::ensure_binary("chrome124", &opts)
        .await
        .map_err(|e| anyhow!("ensure binary failed: {}", e))?;
    let resp = curl_impersonate_cli::Request::get(bin, url)
        .follow_redirects(true)
        .timeout_secs(30.0)
        .send()
        .await
        .map_err(|e| anyhow!("browser get failed: {}", e))?;
    let text = resp.body;
    if text.is_empty() {
        warn!("GET (browser) {} returned empty body", url);
    }
    Ok(text)
}

/// 带浏览器指纹伪装的 GET 请求（支持自定义头）
pub async fn fetch_browser_get_with_headers(
    url: &str,
    headers: &[(&str, &str)],
) -> Result<String, Error> {
    debug!("GET (browser) {} (with headers)", url);
    let opts = curl_impersonate_cli::download::DownloadOptions::default();
    let bin = curl_impersonate_cli::download::ensure_binary("chrome124", &opts)
        .await
        .map_err(|e| anyhow!("ensure binary failed: {}", e))?;
    let mut req = curl_impersonate_cli::Request::get(bin, url)
        .follow_redirects(true)
        .timeout_secs(30.0);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| anyhow!("browser get failed: {}", e))?;
    let text = resp.body;
    if text.is_empty() {
        warn!("GET (browser) {} returned empty body", url);
    }
    Ok(text)
}
