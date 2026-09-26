本文由AI撰写，但其他不是哦！
# 入口函数 - main.rs

## 入口函数

**`src/main.rs:14` — `#[tokio::main] async fn main()`** 是整个程序的入口。

## 功能流程

1. **配置初始化**：调用 `config::init()` 确保 `config.toml` 存在
2. **命令行参数处理**：
   - 第一个参数为 `"docs"` → 调用 `doc_generate()` 生成文档 HTML
   - 第一个参数为 `"cookie"` → 调用 `extract_cookies_to_json()` 导出浏览器 cookie
3. **启动 TCP 服务器**：绑定 `127.0.0.1:配置的端口`（`config.toml` 的 `server.port`，默认 `7878`），使用 `tokio` 异步运行时，`tokio::spawn` 并发处理请求（`Semaphore` 信号量限制并发上限）
4. **请求处理**：每个连接由 `handle_connection()` 处理（定义在 `lib.rs` 的 `connect` 模块）

## 架构图

```
用户 HTTP 请求
    ↓
TcpListener(:7878)  (tokio::spawn 每个连接一个任务)
    ↓  (Semaphore 并发限流)
handle_connection(stream)
    ↓
解析 HTTP 请求头 (GET /xxx?key=val HTTP/1.1)  + keep-alive 处理
    ↓
root_rules(url, params)
    ├── "/"          → show_index_doc().await   → index.html
    ├── "/favicon.ico" → serve_static(...)      → 图标文件
    ├── "/docs/*"    → serve_static(...)        → 文档 HTML
    ├── "/index/*"   → serve_static(...)        → 静态资源
    ├── "/status"    → 内联生成 JSON 状态（路由数/缓存/请求统计，默认关闭，需 [server] status_route = true）
    └── 其他路由       → request_rules(url, params).await
                          ├── 检查 routes.disabled → 禁用则 404
                          ├── 检查 routes.rate_limit → 缓存命中则直接复用
                          └── route_dispatch(url, params).await
                                └── 路由器名::get(params).await  → RSS XML
```

## 环境目录结构

二进制运行所需的目录结构：

```
env/
├── config.toml       # 配置文件（自动创建）
├── cookies.json      # Cookie 文件
├── index/
│   ├── 404.html      # 404 页面
│   ├── favicon.ico   # 站点图标
│   └── index.html    # 首页
├── docs_md/          # Markdown 文档源文件
│   ├── official/     # 官方文档（含自动生成的 routes.md 路由清单）
│   ├── SUMMARY.md    # 导航目录（rssust docs 自动生成）
│   └── 路由名.md     # 各路由的文档
├── docs/             # rssust docs 生成的 HTML 文档
└── rssust            # 编译后的二进制
```

# crawler 模块 — lib.rs（已禁用）

> **注意**：`crawler` 模块目前在 `src/lib.rs` 中已被注释掉（`/* ... */`），当前版本不可用。以下文档仅作历史参考。

## 位置

`src/lib.rs` 中的 `pub mod crawler` 模块（已注释）。

## 概述

管理无头浏览器（Obscura）实例和 Cookie 的生命周期。所有浏览器操作通过线程局部存储（`thread_local!`）实现，每个线程持有独立的浏览器实例。

## 核心类型

### Coke

内部结构体，存储处理后的 Cookie 信息：

```rust
struct Coke {
    url: String,    // 域名
    mai: String,    // Set-Cookie 字符串
}
```

### BROWSER（线程局部变量）

```rust
thread_local! {
    static BROWSER: RefCell<Option<(tokio::runtime::Runtime, Browser)>> = RefCell::new(None);
}
```

每个线程第一次使用时创建 `Runtime` + `Browser` 实例，后续复用。

## 核心函数

### with_browser()

**签名**: `pub fn with_browser<F, T>(f: F) -> Result<T>`

获取当前线程的浏览器实例，执行闭包。如果浏览器尚未初始化，则自动创建：

- 使用 `tokio::runtime::Builder` 创建单线程运行时
- 调用 `Browser::new()` 创建 Obscura 无头浏览器实例

### load_cookies()

**签名**: `pub fn load_cookies() -> Result<String, Error>`

从二进制同目录的 `cookies.json` 加载 Cookie：

1. 读取 JSON 文件（空文件跳过）
2. 反序列化为 `Vec<Value>`（Cookie 对象数组）
3. 对每个 Cookie 调用 `build_set_cookie()` 转换为 `Coke` 结构
4. 调用 `init()` 注入到全局浏览器

### init()

**签名**: `fn init(cookies: &[Coke]) -> Result<()>`

将 Cookie 注入到浏览器实例中，通过 `browser.cookies().set()` 设置。

### build_set_cookie()

**签名**: `fn build_set_cookie(cookie: &Value) -> Result<Coke>`

将 JSON Cookie 对象转换为 HTTP `Set-Cookie` 字符串：

- 必填字段：`name`, `value`, `domain`
- 可选字段：`path`（默认 `/`）, `secure`, `httpOnly`, `sameSite`, `expirationDate`
- `expirationDate` 会计算 `Max-Age` 并附加到字符串

### fetch()

**签名**: `pub fn fetch(url: &str) -> Result<String>`

使用无头浏览器访问指定 URL，返回页面 HTML 内容：

1. 创建新页面（`new_page()`）
2. 导航到 URL（`goto()`）
3. 返回页面内容（`content()`）


# connect 模块 - lib.rs

## 位置

`src/lib.rs` 中的 `pub mod connect` 模块。

## 核心函数

### handle_connection()

**签名**: `pub async fn handle_connection(mut stream: TcpStream)`

这是每个 HTTP 请求的入口处理函数，基于 `tokio` 异步 IO：

1. 用 `read_head()` 从 TCP 流读取完整请求头（直到 `\r\n\r\n`），超过 30 秒无数据返回 `TimedOut`
2. 解析 HTTP 请求行，提取方法、URL 路径（GET 和 POST 之间的部分）
3. 解析 URL 中的查询参数：
   - 如果 URL 包含 `?`，用 `split_once('?')` 分割路径和查询字符串
   - 查询字符串通过 `params_to_hashmap()` 转为 `HashMap`
4. 判断 keep-alive：仅文档/静态路由（`/`、`/docs/*`、`/index/*`、`/favicon.ico`）保持连接，其余动态路由一律 `Connection: close`
5. 获取 `Semaphore` 许可后，用 `tokio::spawn` 将请求任务调度到 worker 线程，调用 `root_rules(path, params).await` 获取响应
6. 根据 `ShowToUser` 枚举类型构建 HTTP 响应：
   - `ShowToUser::Html` → `Content-Type: text/html`
   - `ShowToUser::Rss` → `Content-Type: application/xml`
   - `ShowToUser::File` → 对应的 MIME 类型（css/js/svg/png 等）
7. 写回 TCP 流；keep-alive 下循环读取下一个请求（`buf.drain(..head_len)` 复用缓冲）

### read_head()

**签名**: `async fn read_head(stream: &mut TcpStream, buf: &mut Vec<u8>) -> std::io::Result<Option<usize>>`

读取完整请求头。返回 `Ok(None)` 表示对端干净关闭；`Err(TimedOut)` 表示空闲超时（30 秒）；`Err(UnexpectedEof)` 表示连接中断。

### parse_keep_alive()

**签名**: `fn parse_keep_alive(head: &str, version: &str) -> bool`

解析 `Connection` 头。HTTP/1.1 默认 keep-alive（除非显式 `Connection: close`）；HTTP/1.0 需显式 `Connection: keep-alive`。

### show_index_doc()

**签名**: `pub async fn show_index_doc() -> Result<String, Error>`

读取二进制同目录下的 `index/index.html` 文件，返回 HTML 字符串。

### show_doc()

**签名**: `pub async fn show_doc(path: &str) -> Result<String, Error>`

读取二进制同目录下的文档 HTML 文件（如 `/docs/new_router_cn.html` → 读取 `docs/new_router_cn.html`）。如果文件不存在，返回 `index/404.html`。

## ShowToUser 枚举

定义在 `request_rules.rs` 中，但被 connect 模块使用：

```rust
pub enum ShowToUser {
    Html { res: Result<String, Error> },
    Rss { res: Result<String, Error> },
    File { res: Result<String, Error>, content_type: String },
}
```

- `Html` — 返回 HTML 页面（首页、文档页面、错误页面）
- `Rss` — 返回 RSS XML（路由器的输出）
- `File` — 返回静态文件（CSS/JS/图片/字体等）

# 路由注册表 - request_rules.rs

## 位置

`src/request_rules.rs`

## 概述

这是整个项目的路由调度中心。负责将 URL 路径分发给对应的处理器，并封装返回格式。

## ShowToUser 枚举

```rust
pub enum ShowToUser {
    Html { res: Result<String, Error> },
    Rss { res: Result<String, Error> },
    File { res: Result<String, Error>, content_type: String },
}
```

- `Html` — 返回 HTML 页面
- `Rss` — 返回 RSS XML
- `File` — 返回静态文件

## 核心函数

### root_rules()

**签名**: `pub async fn root_rules(first_part: &str, second_part: HashMap<String, String>) -> ShowToUser`

一级路由分发函数：

| URL 路径 | 处理器 | 返回类型 |
|----------|--------|----------|
| `"/"` | `show_index_doc()` | `ShowToUser::Html` |
| `"/favicon.ico"` | `serve_static("/index/favicon.ico")` | `ShowToUser::File` |
| 以 `"/docs/"` 或 `"/index/"` 开头 | `serve_static(path)` | `ShowToUser::Html` 或 `ShowToUser::File` |
| `"/status"` | 内联拼接 JSON：`{"status":"ok","uptime_secs":...,"routes":ROUTE_COUNT,"cache_entries":...,"requests":...,"failures":...}`。**默认关闭**：未在 `config.toml` 的 `[server]` 设置 `status_route = true` 时返回 404 | `ShowToUser::Html` |
| 其他 | `request_rules()` | `ShowToUser::Rss` 或 `Html(错误)` |

### request_rules()

**签名**: `pub async fn request_rules(url: &str, parameters: HashMap<String, String>) -> Result<String, anyhow::Error>`

二级路由分发入口，先做两层前置检查，再交给 `route_dispatch()`：

1. **禁用检查**：`is_route_disabled(url)` 命中 `config.toml` 的 `routes.disabled` 列表 → 直接返回 `HttpError::not_found("404NotFound")`
2. **限速缓存**：`rate_limit_secs(url)` 读取 `routes.rate_limit` 配置，若有值则开启缓存作用域（`with_cache_scope`）——缓存间隔内的重复请求直接复用上次响应，生成失败时自动清理缓存
3. **分发**：调用 `route_dispatch(url, parameters).await`

### route_dispatch() 与 routes! 宏

`route_dispatch()` 由文件顶部的 `routes!` 宏自动生成。宏展开后定义三个东西：

- `pub const ROUTES: &[&str]` — 全部已注册路由路径的数组（供 `/status` 统计 `ROUTE_COUNT` 等使用）
- `pub const ROUTE_COUNT: usize` — 路由总数
- `pub async fn route_dispatch(url: &str, parameters: HashMap<String, String>) -> Result<String, anyhow::Error>` — 用 `match url` 把路径分发到对应的 `路由器名::get()`

宏观展开后的等价代码形如：

```rust
pub async fn route_dispatch(
    url: &str,
    parameters: HashMap<String, String>,
) -> Result<String, anyhow::Error> {
    match url {
        "/bilibili_weekly" => bilibili_weekly::get(parameters.clone()).await,
        "/bilibili_dynamic" => bilibili_dynamic::get(parameters.clone()).await,
        // ... 其他路由（由 routes! 宏展开）
        _ => {
            warn!("Unregistered route: {}", url);
            Err(HttpError::not_found("404NotFound").into())
        }
    }
}
```

实际代码中不需要手写 `match`，只需在 `request_rules.rs` 的 `routes! { ... }` 块里按字母序加一条 `("/你的路由名", 你的路由名),`，宏会自动生成上述分发逻辑。未注册的路径由 `route_dispatch` 兜底返回 404。

## 如何添加新路由

参考 `new_router_cn.md`，步骤：

1. 在 `src/router/` 下新建 `.rs` 文件，实现 `pub async fn get(para: HashMap<String,String>) -> Result<String, Error>`
2. 在 `src/router/mod.rs` 中添加 `pub mod 你的路由名;`
3. 在 `src/request_rules.rs` 的 `routes! { ... }` 块中按字母序添加 `("/你的路由名", 你的路由名),` 条目（路径与模块名要一致）
4. 在 `env/docs_md/` 下编写对应的路由文档 `.md` 文件
5. 运行 `./env/rssust docs` 重新生成 SUMMARY（自动注册导航）与 `official/routes.md` 路由清单

# 文档生成器 - doc.rs

## 位置

`src/doc.rs`

## 概述

将 `docs_md/` 目录下的 Markdown 文件批量转换为带样式的 HTML 文档页面。在终端中执行 `./env/rssust docs` 触发。

## 核心函数

### doc_generate()

**签名**: `pub fn doc_generate() -> Result<(), Error>`

#### 执行流程

1. **定位目录**：以二进制文件所在目录为基础，输入目录为 `./docs_md`，输出目录为 `./docs`
2. **生成 SUMMARY.md**：自动扫描 `docs_md/` 下的所有 `.md` 文件，按分类生成导航目录（官方文档 → 路由文档）
3. **生成路由清单**：调用 `generate_api_file()` 自动重写 `official/routes.md`——读取每个路由文档的 `**Introduction:**` 字段，按 `#### 分组` 归类输出 `<a>` 链接清单（分组信息取自旧版 `routes.md`，新路由默认归入 `Other`）
4. **三方对账**：调用 `check_route_consistency()` 比对 `src/router/mod.rs` 的 `pub mod` 声明、`src/request_rules.rs` 的 `routes!` 块、`docs_md/路由名.md` 文档三方路由集合，任何缺漏会输出 `DOC-ISS` 日志并使命令返回非零
5. **转换**：使用 `mdbook` 库将 Markdown 转为 HTML
6. **后处理**：
   - 将 `official/` 子目录下的 HTML 提升到 `docs/` 根目录；
   - 修复所有 HTML/JS 中的资源与页面链接路径（`official/` 前缀、`../css`、`../xxx.html` 等，保留 API 示例链接 `../路由名?x=...` 的 `../` 以指向根路由）