# 给用户使用的函数——简便

## 位置

`src/easyuser.rs`

## 概述

封装了一系列辅助函数，供路由开发者（router）方便地调用。所有函数通过 `use crate::easyuser::*;` 导入。

---

## 爬虫类

> **异步**：项目已迁移到 `tokio` 异步运行时，以下 fetch 函数均为 `async fn`，调用时必须加 `.await`，例如 `fetch_reqwest_get(&url).await?`。所有 fetch 函数共享一个全局 `reqwest::Client`（`OnceLock` 懒加载），复用连接池，不会每次新建。
>
> **缓存**：对在 `config.toml` 的 `routes.rate_limit` 中配置了间隔的路由，GET/POST 各变体会自动走 `rate_limit` 模块缓存——间隔内重复请求直接复用缓存、不再请求上游；解析报错时自动清理缓存。
>
> **浏览器指纹**：另有 `fetch_browser_get*` 系列（基于 curl-impersonate-cli）用于绕过 Cloudflare 等按 TLS 指纹反爬的站点，见下文。

### fetch_reqwest_get()

**签名**: `pub async fn fetch_reqwest_get(url: &str) -> Result<String, Error>`

使用 `reqwest` 库发送 HTTP GET 请求，返回响应文本。不携带 Cookie，不执行 JavaScript。

### fetch_reqwest_get_with_headers()

**签名**: `pub async fn fetch_reqwest_get_with_headers(url: &str, headers: &[(&str, &str)]) -> Result<String, Error>`

使用 `reqwest` 库发送带自定义 Header 的 HTTP GET 请求。

示例：
```rust
let headers = &[("Referer", "https://example.com"), ("Cookie", "session=abc")];
fetch_reqwest_get_with_headers("https://api.example.com/data", headers).await?;
```

### fetch_reqwest_post()

**签名**: `pub async fn fetch_reqwest_post(url: &str, body: String, header: Option<reqwest::header::HeaderMap>) -> Result<String, Error>`

使用 `reqwest` 库发送 HTTP POST 请求，返回响应文本。第三个参数 `header` 用于自定义请求头，不需要时传 `None`。

### fetch_reqwest_post_json()

**签名**: `pub async fn fetch_reqwest_post_json(url: &str, json_body: &str) -> Result<String, Error>`

使用 `reqwest` 库发送 Content-Type 为 `application/json` 的 HTTP POST 请求。

### fetch_reqwest_post_json_with_headers()

**签名**: `pub async fn fetch_reqwest_post_json_with_headers(url: &str, json_body: &str, headers: &[(&str, &str)]) -> Result<String, Error>`

发送 Content-Type 为 `application/json` 的 POST 请求，并携带自定义 Headers。

### client_with_cookie()

**签名**: `pub fn client_with_cookie() -> Result<reqwest::Client, Error>`

构建一个启用了 `cookie_store(true)` 的 `reqwest::Client` 实例（独立于全局共享 Client），用于需要带 Cookie 维持会话的场景。

### fetch_browser_get()

**签名**: `pub async fn fetch_browser_get(url: &str) -> Result<String, Error>`

带浏览器 TLS 指纹伪装的 GET 请求（绕过 Cloudflare 等反爬），默认使用 `chrome124` profile。

### fetch_browser_get_with_profile()

**签名**: `pub async fn fetch_browser_get_with_profile(url: &str, profile: &str) -> Result<String, Error>`

同上，但可指定 curl-impersonate 的 profile（如 `chrome110` / `edge99`，某些站点仅对特定旧指纹放行）。

### fetch_browser_get_with_headers()

**签名**: `pub async fn fetch_browser_get_with_headers(url: &str, headers: &[(&str, &str)]) -> Result<String, Error>`

带浏览器指纹伪装，并携带自定义 Headers 的 GET 请求（默认 `chrome124`）。

### fetch_browser_get_with_headers_profile()

**签名**: `pub async fn fetch_browser_get_with_headers_profile(url: &str, headers: &[(&str, &str)], profile: &str) -> Result<String, Error>`

带浏览器指纹伪装 + 自定义 Headers + 指定 profile 的 GET 请求。

> **注意**：`fetch_browser_get*` 依赖 `curl-impersonate-cli`（已启用其 `download` feature），首次调用会自动下载预编译的 curl-impersonate 二进制，之后复用。

### HttpError

**签名**: `pub struct HttpError { pub status: u16, pub message: String }`

自定义错误类型，可携带 HTTP 状态码。路由中可用 `HttpError::bad_request(msg)`、`HttpError::not_found(msg)`、`HttpError::bad_gateway(msg)` 构造。渲染层会优先使用 `HttpError` 的 `status` 作为响应状态码、`message` 作为正文（否则默认 500）。

---

## 序列化类

### req_param()

**签名**: `pub fn req_param(params: &HashMap<String, String>, key: &str, desc: &str) -> Result<String, Error>`

取必填参数：存在且非空时返回其值，否则返回错误 `Params should contain key (desc)`。

### req_optional()

**签名**: `pub fn req_optional(params: &HashMap<String, String>, key: &str) -> Option<String>`

取可选参数，缺失返回 `None`。

### req_usize()

**签名**: `pub fn req_usize(params: &HashMap<String, String>, key: &str, desc: &str) -> Result<usize, Error>`

取必填参数并要求解析为 `usize`，失败返回错误说明。

### opt_usize()

**签名**: `pub fn opt_usize(params: &HashMap<String, String>, key: &str, default: usize) -> usize`

取可选整数参数，缺失或解析失败返回 `default`。

### params_to_hashmap()

**签名**: `pub fn params_to_hashmap(query: &str) -> HashMap<String, String>`

将 `key1=value1&key2=value2` 格式参数字符串解析为 `HashMap`。

**用在**: `connect::handle_connection()` 中解析 URL 查询参数。

### hashmap_to_params()

**签名**: `pub fn hashmap_to_params(hashmap: HashMap<String, String>) -> String`

将 `HashMap` 序列化为 `key1=value1&key2=value2` 格式字符串。

---

## 时间类

### now()

**签名**: `pub fn now() -> String`

返回当前时间的 RSS 标准格式：`Sat, 11 Jul 2026 12:00:00 +0800`

**用在**: 路由中设置 RSS Item 的 `pub_date` 字段。

### chinese_date_to_parse()

**签名**: `pub fn chinese_date_to_parse(input: &str) -> Option<String>`

将中文日期格式 `x月y日` 解析为 RSS 标准时间格式。年份取当前年份。

示例：`"7月11日"` → `"Sat, 11 Jul 2026 00:00:00 +0800"`

### timestamp_to_rss()

**签名**: `pub fn timestamp_to_rss(ts: i64) -> String`

将 Unix 时间戳（i64）转换为 RSS 标准时间格式。

### datetime_str_to_rss()

**签名**: `pub fn datetime_str_to_rss(datetime_str: &str) -> Option<String>`

将 `"YYYY-MM-DD HH:MM:SS"` 格式的字符串转换为 RSS 标准时间格式。**输入视为东八区本地时间**（不做偏移，直接附加 +0800）。

示例：`"2026-07-11 12:00:00"` → `"Sat, 11 Jul 2026 12:00:00 +0800"`

注意：若来源时间是 UTC，请改用 `utc_str_to_rss`。

### utc_str_to_rss()

**签名**: `pub fn utc_str_to_rss(datetime_str: &str) -> Option<String>`

将 `"YYYY-MM-DD HH:MM:SS"` 格式的字符串转换为 RSS 标准时间格式。**输入视为 UTC 时间**，自动换算为东八区（+0800）后再输出。

示例：`"2026-07-11 12:00:00"` → `"Sat, 11 Jul 2026 20:00:00 +0800"`

适用于带 `Z` 后缀的 ISO 时间（如 `datePublished`），调用前先 `s.replace('T', " ").replace('Z', "")` 归一化，参见 `apnews_topics` 的用法。

### rfc3339_to_rss()

**签名**: `pub fn rfc3339_to_rss(s: &str) -> String`

将 RFC 3339 格式字符串（如 `2026-07-11T12:00:00+08:00`）转换为 RSS 标准时间格式。

### rfc3339_to_rss_utc()

**签名**: `pub fn rfc3339_to_rss_utc(s: &str) -> String`

同上，但输入视为 UTC 时间，输出换算为东八区（+0800）。

### rfc2822_to_rss()

**签名**: `pub fn rfc2822_to_rss(s: &str) -> String`

将 RFC 2822 格式（如 `Sat, 11 Jul 2026 12:00:00 +0800`）转换为 RSS 标准时间格式。

### date_str_to_rss()

**签名**: `pub fn date_str_to_rss(date_str: &str, fmt: &str, offset: &str) -> Option<String>`

按自定义 `fmt` 解析日期字符串，并按 `offset`（如 `+0800`）校正时区后输出 RSS 标准时间格式。

---

## 字符类

### no_double_quotes()

**签名**: `pub fn no_double_quotes(s: String) -> String`

去除字符串首尾的双引号。

**用在**: 路由中清理从 JSON 提取的字符串值，因为 `serde_json::Value::to_string()` 会给字符串加双引号。

### env_search()

**签名**: `pub fn env_search(s: &str) -> Option<String>`

查找环境变量值，找到返回 `Some(val)`，找不到返回 `None`。

---

## 工具类

### parse_bool()

**签名**: `pub fn parse_bool(value: Option<&String>, default: bool) -> bool`

将字符串值解析为布尔值。支持 `"1"` / `"true"` / `"True"` → `true`，`"0"` / `"false"` / `"False"` → `false`，其他值尝试 `parse()`，`None` 返回 `default`。

### load_cookie_header()

**签名**: `pub fn load_cookie_header(domain_filter: Option<&str>) -> Result<Option<String>>`

从二进制同目录的 `cookies.json` 加载 Cookie，返回 `name=value; name=value...` 格式的字符串。可传入 `domain_filter` 只返回指定域名的 Cookie（如 `Some("bilibili.com")`）。

### resolve_url()

**签名**: `pub fn resolve_url(href: &str, base: &str) -> String`

将相对链接解析为绝对 URL（`base` 为页面地址）。支持 `//` 协议相对、`/` 根相对、`../` 等形式的解析。

### truncate()

**签名**: `pub fn truncate(s: &str, n: usize) -> String`

将字符串截断到最多 `n` 个字符（按 `char` 计数，避免截断多字节 UTF-8），超出部分以单个省略号 `…` 结尾。

---

## 开发者提示

- 路由中必须导入 `use crate::easyuser::*;`
- 从 JSON 提取字符串后记得用 `no_double_quotes()` 清理
- 需要携带 Cookie 时，用 `fetch_reqwest_get_with_headers()` 配合 `load_cookie_header()`
- **所有 fetch 调用都要加 `.await`**（`async fn`），且不要在持有 scraper 的 `Html`/`Selector`/`ElementRef` 时跨 `.await`（它们不是 `Send`），详见 `new_router_cn.md` 的同步转异步章节