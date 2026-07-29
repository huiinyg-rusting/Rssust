# 给用户使用的函数——简便

## 位置

`src/easyuser.rs`

## 概述

封装了一系列辅助函数，供路由开发者（router）方便地调用。所有函数通过 `use crate::easyuser::*;` 导入。

---

## 爬虫类

### fetch_reqwest_get()

**签名**: `pub fn fetch_reqwest_get(url: &str) -> Result<String, Error>`

使用 `reqwest` 库发送 HTTP GET 请求，返回响应文本。不携带 Cookie，不执行 JavaScript。

**注意**: 每次调用都会创建新的 Tokio Runtime，开销较大。

### fetch_reqwest_get_with_headers()

**签名**: `pub fn fetch_reqwest_get_with_headers(url: &str, headers: &[(&str, &str)]) -> Result<String, Error>`

使用 `reqwest` 库发送带自定义 Header 的 HTTP GET 请求。

示例：
```rust
let headers = &[("Referer", "https://example.com"), ("Cookie", "session=abc")];
fetch_reqwest_get_with_headers("https://api.example.com/data", headers)?;
```

### fetch_reqwest_post()

**签名**: `pub fn fetch_reqwest_post(url: &str, body: String) -> Result<String, Error>`

使用 `reqwest` 库发送 HTTP POST 请求，返回响应文本。

---

## 序列化类

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

将 `"YYYY-MM-DD HH:MM:SS"` 格式的字符串转换为 RSS 标准时间格式（东八区）。

示例：`"2026-07-11 12:00:00"` → `"Sat, 11 Jul 2026 12:00:00 +0800"`

---

## 字符类

### no_double_quotes()

**签名**: `pub fn no_double_quotes(s: String) -> String`

去除字符串首尾的双引号。

**用在**: 路由中清理从 JSON 提取的字符串值，因为 `serde_json::Value::to_string()` 会给字符串加双引号。

### env_search()

**签名**: `pub fn env_search(s: &str) -> Option<String>`

查找环境变量值，找到返回 `Some(val)`，找不到返回 `None`。

### extract_js_var()

**签名**: `pub fn extract_js_var(html: &str, var_name: &str) -> Result<Value, Error>`

从 HTML 页面中提取 `window.xxx = {...};` 格式的 JavaScript 变量，解析为 JSON `Value`。

---

## 工具类

### parse_bool()

**签名**: `pub fn parse_bool(value: Option<&String>, default: bool) -> bool`

将字符串值解析为布尔值。支持 `"1"` / `"true"` / `"True"` → `true`，`"0"` / `"false"` / `"False"` → `false`，其他值尝试 `parse()`，`None` 返回 `default`。

### load_cookie_header()

**签名**: `pub fn load_cookie_header(domain_filter: Option<&str>) -> Result<Option<String>>`

从二进制同目录的 `cookies.json` 加载 Cookie，返回 `name=value; name=value...` 格式的字符串。可传入 `domain_filter` 只返回指定域名的 Cookie（如 `Some("bilibili.com")`）。

---

## 开发者提示

- 路由中必须导入 `use crate::easyuser::*;`
- 从 JSON 提取字符串后记得用 `no_double_quotes()` 清理
- 需要携带 Cookie 时，用 `fetch_reqwest_get_with_headers()` 配合 `load_cookie_header()`