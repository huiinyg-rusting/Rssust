本文由AI撰写，但其他不是哦！
# 入口函数 - main.rs

## 入口函数

**`src/main.rs:9` — `#[tokio::main] async fn main()`** 是整个程序的入口。

## 功能流程

1. **配置初始化**：调用 `config::init()` 确保 `config.toml` 存在
2. **命令行参数处理**：
   - 第一个参数为 `"docs"` → 调用 `doc_generate()` 生成文档 HTML
   - 第一个参数为 `"cookie"` → 调用 `extract_cookies_to_json()` 导出浏览器 cookie
3. **启动 TCP 服务器**：绑定 `127.0.0.1:7878`，使用 `tokio` 异步运行时，`tokio::spawn` 并发处理请求（`Semaphore` 信号量限制并发上限）
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
    ├── "/"        → show_index_doc().await   → index.html
    ├── "/docs/*"  → show_doc(path).await     → 文档 HTML
    └── 其他路由    → request_rules(url, params).await
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
│   └── index.html    # 首页
├── docs_md/          # Markdown 文档源文件
│   ├── official/     # 官方文档
│   └── 路由名.md     # 各路由的文档
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

**签名**: `pub fn root_rules(first_part: &str, second_part: HashMap<String, String>) -> ShowToUser`

一级路由分发函数：

| URL 路径 | 处理器 | 返回类型 |
|----------|--------|----------|
| `"/"` | `show_index_doc()` | `ShowToUser::Html` |
| 以 `"/docs/"` 或 `"/index/"` 开头 | `serve_static(path)` | `ShowToUser::Html` 或 `ShowToUser::File` |
| 其他 | `request_rules()` | `ShowToUser::Rss` 或 `Html(错误)` |

### request_rules()

**签名**: `pub fn request_rules(url: &str, parameters: HashMap<String, String>) -> Result<String, Error>`

二级路由注册表，使用 `const` 静态数组匹配 URL 路径到具体路由器：

```rust
pub fn request_rules(url: &str, parameters: HashMap<String, String>) -> Result<String, Error> {
    const ROUTES: &[(&str, fn(HashMap<String, String>) -> Result<String, Error>)] = &[
        ("/bilibili_weekly", bilibili_weekly::get),
        ("/bilibili_dynamic", bilibili_dynamic::get),
        // ... 其他路由
    ];
    if let Some(handler) = ROUTES.iter().find(|(path, _)| *path == url) {
        if is_route_disabled(url) {
            return Err(anyhow!("404NotFound"));
        }
        (handler.1)(parameters)
    } else {
        Err(anyhow!("404NotFound"))
    }
}
```

注册的路由会先检查 `config.toml` 中的 `routes.disabled` 列表，若被禁用则返回 404。

## 如何添加新路由

参考 `new_router_cn.md`，步骤：

1. 在 `src/router/` 下新建 `.rs` 文件，实现 `pub async fn get(para: HashMap<String,String>) -> Result<String, Error>`
2. 在 `src/router/mod.rs` 中添加 `pub mod 你的路由名;`
3. 在 `request_rules.rs` 的 `match url` 分发中添加 `"/你的路由名" => run!(你的路由名, parameters)` 条目
4. 在 `env/docs_md/` 下编写对应的路由文档 `.md` 文件

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
2. **生成 SUMMARY.md**：自动扫描 `docs_md/` 下的所有 `.md` 文件，按分类生成导航目录
3. **转换**：使用 `mdbook` 库将 Markdown 转为 HTML
4. **后处理**：将 `official/` 子目录下的 HTML 提升到 `docs/` 根目录，修复所有资源文件和页面链接的路径