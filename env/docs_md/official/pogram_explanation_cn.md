本文由AI撰写，但其他不是哦！
# 入口函数 - main.rs

## 入口函数

**`src/main.rs:9` — `fn main()`** 是整个程序的入口。

## 功能流程

1. **命令行参数处理**：如果第一个参数为 `"docs"`，则调用 `doc_generate()` 生成文档 HTML（遍历 `docs_md/` 下的 `.md` 文件，输出为 `.html` 到 `docs/` 目录）
2. **Cookie 加载**：调用 `load_cookies()` 从二进制同目录的 `cookies.json` 加载 cookie，注入到无头浏览器实例
3. **启动 TCP 服务器**：绑定 `127.0.0.1:7878`，使用 `ThreadPool`（4 线程）并发处理请求
4. **请求处理**：每个连接由 `handle_connection()` 处理（定义在 `lib.rs` 的 `connect` 模块）

## 架构图

```
用户 HTTP 请求
    ↓
TcpListener(:7878)
    ↓  (ThreadPool 分配线程)
handle_connection(stream)
    ↓
解析 HTTP 请求行 (GET /xxx?key=val HTTP/1.1)
    ↓
root_rules(url, params)
    ├── "/"        → show_index_doc()       → index.html
    ├── "/docs/*"  → show_doc(path)          → 文档 HTML
    └── 其他路由    → request_rules(url, params)
                      └── 路由器名::get(params)  → RSS XML
```

## 环境目录结构

二进制运行所需的目录结构：

```
env/
├── cookies.json      # Cookie 文件，启动时加载到浏览器
├── index/
│   ├── 404.html      # 404 页面
│   └── index.html    # 首页
├── docs_md/          # Markdown 文档源文件
│   ├── official/     # 官方文档
│   └── 路由名.md     # 各路由的文档
└── rssust            # 编译后的二进制
```
# crawler 模块 - lib.rs

## 位置

`src/lib.rs` 中的 `pub mod crawler` 模块。

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

**签名**: `pub fn handle_connection(mut stream: TcpStream)`

这是每个 HTTP 请求的入口处理函数：

1. 从 TCP 流读取 1024 字节缓冲区
2. 用 `extract_between_spaces()` 提取 HTTP 请求行中的 URL 路径（GET 和 POST 之间的部分）
3. 解析 URL 中的查询参数：
   - 如果 URL 包含 `?`，用 `split_once('?')` 分割路径和查询字符串
   - 查询字符串通过 `params_to_hashmap()` 转为 `HashMap`
4. 调用 `root_rules(path, params)` 获取响应
5. 根据 `ShowToUser` 枚举类型构建 HTTP 响应：
   - `ShowToUser::Html` → `Content-Type: text/html`
   - `ShowToUser::Rss` → `Content-Type: application/xml`
6. 写回 TCP 流

### extract_between_spaces()

**签名**: `fn extract_between_spaces(buffer: &[u8; 1024]) -> Option<&[u8]>`

从 HTTP 原始请求中提取两个空格之间的内容（即 URL 路径）。例如从 `GET /bilibili_weekly?key=val HTTP/1.1` 中提取 `/bilibili_weekly?key=val`。

### show_index_doc()

**签名**: `pub fn show_index_doc() -> Result<String, Error>`

读取二进制同目录下的 `index/index.html` 文件，返回 HTML 字符串。

### show_doc()

**签名**: `pub fn show_doc(path: &str) -> Result<String, Error>`

读取二进制同目录下的文档 HTML 文件（如 `/docs/new_router_cn.html` → 读取 `docs/new_router_cn.html`）。如果文件不存在，返回 `index/404.html`。

## ShowToUser 枚举

定义在 `request_rules.rs` 中，但被 connect 模块使用：

```rust
pub enum ShowToUser {
    Html { res: Result<String, Error> },
    Rss { res: Result<String, Error> },
}
```

- `Html` — 返回 HTML 页面（首页、文档页面、错误页面）
- `Rss` — 返回 RSS XML（路由器的输出）

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
}
```

- `Html` — 返回 HTML 页面
- `Rss` — 返回 RSS XML

## 核心函数

### root_rules()

**签名**: `pub fn root_rules(first_part: &str, second_part: HashMap<String, String>) -> ShowToUser`

一级路由分发函数：

| URL 路径 | 处理器 | 返回类型 |
|----------|--------|----------|
| `"/"` | `show_index_doc()` | `ShowToUser::Html` |
| `以 "/docs/" 开头` | `show_doc(path)` | `ShowToUser::Html` |
| 其他 | `request_rules()` | `ShowToUser::Rss` 或 `Html(错误)` |

### request_rules()

**签名**: `pub fn request_rules(url: &str, parameters: HashMap<String, String>) -> Result<String, Error>`

二级路由注册表，匹配 URL 路径到具体路由器：

```rust
pub fn request_rules(url: &str, parameters: HashMap<String, String>) -> Result<String, Error> {
    if url == "/bilibili_weekly" {
        bilibili_weekly::get(parameters)
    } else {
        Err(anyhow!("404NotFound"))
    }
}
```

**注意**：`bilibili_dynamic` 路由虽然在 `mod.rs` 中注册了 `pub mod bilibili_dynamic`，但当前 **未在 `request_rules()` 中添加匹配分支**，因此访问 `/bilibili_dynamic` 会返回 404。

## 如何添加新路由

参考 `new_router_cn.md`，步骤：

1. 在 `src/router/` 下新建 `.rs` 文件，实现 `pub fn get(para: HashMap<String,String>) -> Result<String, Error>`
2. 在 `src/router/mod.rs` 中添加 `pub mod 你的路由名;`
3. 在 `request_rules()` 中添加 `else if` 分支匹配 URL
4. 在 `env/docs_md/` 下编写对应的路由文档 `.md` 文件

# 文档生成器 - doc.rs

## 位置

`src/doc.rs`

## 概述

将 `docs_md/` 目录下的 Markdown 文件批量转换为带样式的 HTML 文档页面。在 `src/main.rs` 中通过 `cargo run docs` 触发。

## 核心函数

### doc_generate()

**签名**: `pub fn doc_generate() -> Result<(), Error>`

#### 执行流程

1. **定位目录**：以二进制文件所在目录为基础，输入目录为 `./docs_md`，输出目录为 `./docs`
2. **收集文件**：mdbook干的事
3. **排序**：按文件名（字母顺序，不区分大小写）排序
4. **分类**：`docs_md/official/` 下的文件归为"官方"类，其余归为"Router"类，分别生成导航链接
5. **转换**：使用 `mdbook` 库将 Markdown 转为 HTML
6. **输出**：写入 `docs/目录名.html`

#### 页面功能

- **侧边栏导航**：可折叠/展开，支持关键词搜索过滤目录
- **文章搜索**：实时高亮文章中的匹配文本
- **代码高亮**：支持 Rust、Python、JavaScript、Bash、XML 语言
- **响应式设计**：深色主题，自适应布局

#### 样式特点

- 深紫色渐变背景
- 代码块深色底 + 彩色高亮
- 胶囊形状的"当前页面"指示器
- 紫蓝色点缀色