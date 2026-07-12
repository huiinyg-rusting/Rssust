# 给用户使用的函数——简便

## 位置

`src/easyuser.rs`

## 概述

封装了一系列辅助函数，供路由开发者（router）方便地调用。所有函数通过 `use crate::easyuser::*;` 导入。

---

## 爬虫类

### fetch_obscura()

**签名**: `pub fn fetch_obscura(url: &str) -> Result<String, Error>`

使用无头浏览器（Obscura）爬取 URL 内容。会自动携带 `cookies.json` 中注入的 Cookie。适用于需要 JavaScript 渲染的页面。

**内部调用**: `crawler::fetch(url)`

### fetch_reqwest_get()

**签名**: `pub fn fetch_reqwest_get(url: &str) -> Result<String, Error>`

使用 `reqwest` 库发送 HTTP GET 请求，返回响应文本。不携带 Cookie，不执行 JavaScript。

**注意**: 每次调用都会创建新的 Tokio Runtime，开销较大。

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

## 开发者提示

- 路由中必须导入 `use crate::easyuser::*;`
- `fetch_obscura` 比 `fetch_reqwest_get` 慢（需要启动浏览器），但能处理 JS 渲染的页面
- 从 JSON 提取字符串后记得用 `no_double_quotes()` 清理
