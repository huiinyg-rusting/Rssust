pub mod easyuser;
pub mod request_rules;
pub mod router;
use anyhow::*;
use comrak::{markdown_to_html, ComrakOptions};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use std::env;

///这个函数提供缓冲区的处理
/// 并把数据交给request_rules函数处理
/// 最终在这个函数体内发送http数据
pub mod connect {
    use anyhow::Error;
    use anyhow::{Result, anyhow};

    use crate::request_rules::*;
    use std::collections::HashMap;
    use std::fs;
    use std::io::prelude::*;
    use std::net::TcpStream;
    use std::path::Path;

    pub fn handle_connection(mut stream: TcpStream) {
        let mut buffer: [u8; 1024] = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let head = std::str::from_utf8(extract_between_spaces(&buffer).unwrap_or_else(|| {
            eprintln!("Failed to slice header{:?}", buffer);
            &[]
        }))
        .unwrap_or_else(|_| {
            eprintln!("Invalid UTF-8{:?}", buffer);
            ""
        });
        let response = if head.contains("?") {
            if let Some((before, after)) = head.split_once('?') {
                let first_part = before;
                let second_part = crate::easyuser::params_to_hashmap(after); //执行完second_part此时已经是hashmap格式了
                root_rules(first_part, second_part)
            } else {
                root_rules(head, HashMap::new())
            }
        } else {
            root_rules(head, HashMap::new())
        };
        let response = match response {
            ShowToUser::Html {res} => {
                match res {
                Ok(i) => format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                    i.len(),
                    i
                ),
                Err(i) => format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i),
            }
            },
            ShowToUser::Rss { res } => {
                match res {
                Ok(i) => format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/xml; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                    i.len(),
                    i
                ),
                Err(i) => format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i),
            }
            }
        };

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn extract_between_spaces(buffer: &[u8; 1024]) -> Option<&[u8]> {
        let bytes = buffer.as_slice();

        // 找到第一个空格的位置
        let first_space = bytes.iter().position(|&b| b == b' ')?;

        // 从第一个空格之后开始，找第二个空格
        let second_space = bytes[first_space + 1..]
            .iter()
            .position(|&b| b == b' ')
            .map(|pos| first_space + 1 + pos)?;

        // 截取两个空格之间的内容
        Some(&bytes[first_space + 1..second_space])
    }

    ///这个函数发送index.html的内容给调用者，否则发送错误及anyhow的文本错误类型 给调用者
    /// 给调用者的是html格式
    pub fn show_index_doc() -> Result<String, Error> {
        match fs::read_to_string(&Path::new("index/index.html")) {
            Ok(i) => Ok(i),

            Err(i) => Err(anyhow!(format!("{}:{}", "index.html", i.kind()))),
        }
    }
}

pub mod crawler {
    use anyhow::{Error, Ok, Result, anyhow};
    use obscura::Browser;
    use serde_json::Value;
    use std::cell::RefCell;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::runtime::Builder as RuntimeBuilder;

    #[derive(Debug)]
    struct Coke {
        url: String,
        mai: String,
    }

    thread_local! {
        static BROWSER: RefCell<Option<(tokio::runtime::Runtime, Browser)>> = RefCell::new(None);
    }

    fn with_browser<F, T>(f: F) -> Result<T>
    where
        F: FnOnce(&tokio::runtime::Runtime, &mut Browser) -> Result<T>,
    {
        BROWSER.with(|cell| {
            let mut guard = cell.borrow_mut();
            if guard.is_none() {
                let rt = RuntimeBuilder::new_current_thread().enable_all().build()?;
                let browser = rt.block_on(async { Browser::new() })?;
                *guard = Some((rt, browser));
            }
            let (rt, browser) = guard.as_mut().unwrap();
            f(rt, browser)
        })
    }

    /// 将一个 JSON cookie 对象转换为 HTTP Set-Cookie 字符串。
    ///
    /// 示例格式：
    /// {
    ///   "name": "session_id",
    ///   "value": "abc123",
    ///   "domain": ".example.com",
    ///   "path": "/",
    ///   "secure": true,
    ///   "httpOnly": true,
    ///   "sameSite": "Lax",
    ///   "expirationDate": 1893456000
    /// }
    fn build_set_cookie(cookie: &Value) -> Result<Coke> {
        let name = cookie
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("cookie 缺少 name 字段"))?;
        let value = cookie
            .get("value")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("cookie 缺少 value 字段"))?;
        let domain = cookie
            .get("domain")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("cookie 缺少 domain 字段"))?;
        let path = cookie.get("path").and_then(Value::as_str).unwrap_or("/");

        let mut set_cookie = format!("{}={}; Domain={}; Path={}", name, value, domain, path);
        if cookie
            .get("secure")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            set_cookie.push_str("; Secure");
        }
        if cookie
            .get("httpOnly")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            set_cookie.push_str("; HttpOnly");
        }
        if let Some(same_site) = cookie.get("sameSite").and_then(Value::as_str) {
            if !same_site.is_empty() && same_site != "None" {
                set_cookie.push_str(&format!("; SameSite={}", same_site));
            }
        }

        if let Some(expiration) = cookie.get("expirationDate").and_then(Value::as_f64) {
            if let std::result::Result::Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                let max_age = expiration as i64 - now.as_secs() as i64;
                if max_age > 0 {
                    set_cookie.push_str(&format!("; Max-Age={}", max_age));
                }
            }
        }

        Ok(Coke {
            url: (domain.to_string()),
            mai: (set_cookie),
        })
    }

    /// 从 JSON 文件加载 cookie，返回 Set-Cookie 字符串列表。
    /// JSON 文件应为 cookie 对象数组。
    pub fn load_cookies() -> Result<String, Error> {
        let exe_path = env::current_exe()?;
    

        let exe_dir = exe_path.parent()
            .ok_or_else(|| anyhow!("Could not get executable directory"))?;
            
        let raw = exe_dir.join("cookies.json");
        let text = std::fs::read_to_string(raw)?;
        if text.is_empty() {return  Ok("".to_string());};
        let cookies: Vec<Value> = serde_json::from_str(&text)?;
        let coke_list: Vec<Coke> = cookies
            .iter()
            .map(build_set_cookie)
            .collect::<Result<Vec<_>, _>>()?;

        let _ = init(&coke_list);
        println!("成功载入 {} 个 Cookie", cookies.len());
        Ok("Succse".to_string())
    }

    /// 将 cookie 注入全局浏览器实例。
    fn init(cookies: &[Coke]) -> Result<()> {
        with_browser(|_rt, browser| {
            for cmake in cookies {
                browser.cookies().set(
                    cmake.mai.as_str(),
                    format!("https://www{}", cmake.url).as_str(),
                )?;
            }
            Ok(())
        })
    }
    pub fn fetch(url: &str) -> Result<String> {
        with_browser(|rt, browser| {
            rt.block_on(async {
                let mut page = browser.new_page().await?;
                page.goto(url).await?;
                Ok(page.content())
            })
        })
    }
}

pub fn doc_generate() -> Result<(), Error> {
    //这个函数是AI写的，我再修改，从试验项目迁到这，不要骂我
    let exe_path = env::current_exe()?;


    let exe_dir = exe_path.parent()
        .ok_or_else(|| anyhow!("Could not get executable directory"))?;
        
    let input_dir = exe_dir.join("./docs_md");
    let output_dir = exe_dir.join("./docs");
    fs::create_dir_all(&output_dir)?;

    // 1. 收集所有 md 文件路径
    let mut md_files_all = Vec::new();
    for entry in WalkDir::new(&input_dir) {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
            md_files_all.push(entry.path().to_path_buf());
        }
    }

    // 2. 分离 official 和 router 的文件链接
    let official_dir = std::path::Path::new(&input_dir).join("official");
    let mut official_links = String::new();
    let mut router_links = String::new();

    for path in &md_files_all {
        let file_name = path.file_stem().unwrap().to_str().unwrap();
        let link = format!("<li><a href='{}.html'>{}</a></li>", file_name, file_name);
        if path.starts_with(&official_dir) {
            official_links.push_str(&link);
        } else {
            router_links.push_str(&link);
        }
    }

    // 3. 逐个转换
    for path in &md_files_all {
        let content = fs::read_to_string(path)?;
        let file_name = path.file_stem().unwrap().to_str().unwrap();

        // 渲染 Markdown
        let html_body = markdown_to_html(&content, &ComrakOptions::default());

        // 组装完整 HTML
        let full_html = format!(
            r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            min-height: 100vh;
            background: linear-gradient(135deg, #0f0c29, #302b63, #24243e);
            color: #e0e0e0;
        }}

        .page-shell {{ display: flex; min-height: 100vh; }}
        .current-page-badge {{
            display: inline-flex;
            align-items: center;
            
            /* 放大关键属性 */
            padding: 10px 15px;      /* 更大的内边距 */
            font-size: 1.2rem;       /* 明显放大的字体 */
            font-weight: 600;
            
            /* 视觉风格 */
            background-color: #7b2fff59; /* 亮蓝色背景 */
            color: white;              /* 白色文字 */
            border-radius: 50px;       /* 完全圆角胶囊形状 */
            box-shadow: 0 4px 6px rgba(86, 9, 145, 0.3); /* 彩色阴影 */
        }}
        .sidebar {{
            flex: 0 0 260px;
            width: 260px;
            background: rgba(10, 5, 20, 0.96);
            padding: 22px;
            border-right: 1px solid #7b2fff59;
            overflow-y: auto;
            position: sticky;
            top: 0;
            height: 100vh;
            transition: flex-basis 0.25s ease, width 0.25s ease, padding 0.25s ease, border-right 0.25s ease;
        }}

        .sidebar.collapsed {{
            flex: 0 0 0;
            width: 0;
            padding-left: 0;
            padding-right: 0;
            border-right: none;
            overflow: hidden;
        }}

        .sidebar h3 {{
            font-size: 16px;
            color: #b026ff;
            margin-bottom: 16px;
            display: flex;
            justify-content: space-between;
            align-items: center;
            gap: 10px;
        }}

        .sidebar-section-title {{
            font-size: 13px;
            color: #8888cc;
            text-transform: uppercase;
            letter-spacing: 0.08em;
            margin: 18px 0 8px 0;
            padding-bottom: 4px;
            border-bottom: 1px solid rgba(123, 47, 255, 0.2);
        }}

        .sidebar ul {{ list-style: none; padding: 0; }}
        .sidebar li {{ margin-bottom: 8px; }}
        .sidebar a {{
            display: block;
            padding: 10px 14px;
            border-radius: 8px;
            color: #c9a0ff;
            text-decoration: none;
            transition: background 0.2s ease, color 0.2s ease;
        }}
        .sidebar a:hover {{
            background: rgba(123, 47, 255, 0.18);
            color: #fff;
        }}

        .sidebar-search {{
            width: 100%;
            padding: 8px 12px;
            border: 1px solid rgba(123, 47, 255, 0.3);
            border-radius: 8px;
            background: rgba(0, 0, 0, 0.3);
            color: #e0e0e0;
            font-size: 13px;
            margin-bottom: 12px;
            outline: none;
        }}
        .sidebar-search:focus {{
            border-color: #b026ff;
        }}
        .sidebar-search::placeholder {{
            color: #6666aa;
        }}

        .main-container {{
            flex: 1;
            display: flex;
            flex-direction: column;
            min-height: 100vh;
            width: auto;
            transition: width 0.25s ease;
        }}

        .main-container.expanded {{
            width: auto;
        }}

        .top-bar {{
            padding: 18px 24px;
            background: rgba(0, 0, 0, 0.45);
            border-bottom: 1px solid rgba(123, 47, 255, 0.3);
            display: flex;
            align-items: center;
            gap: 16px;
            position: sticky;
            top: 0;
            z-index: 5;
        }}

        .expand-btn {{
            padding: 9px 16px;
            border: none;
            border-radius: 8px;
            background: #7b2fff;
            color: #fff;
            cursor: pointer;
            transition: background 0.2s ease;
        }}
        .expand-btn:hover {{ background: #b026ff; }}

        .brand {{
            font-size: 20px;
            font-weight: 700;
            color: #b026ff;
            text-decoration: none;
            margin-right: 4px;
        }}
        .brand:hover {{ color: #d2a8ff; }}

        .nav-link {{
            color: #c9a0ff;
            text-decoration: none;
            font-size: 14px;
            padding: 6px 12px;
            border-radius: 6px;
            transition: background 0.2s;
        }}
        .nav-link:hover {{ background: rgba(123, 47, 255, 0.18); color: #fff; }}

        .top-bar .spacer {{ flex: 1; }}

        .article-search {{
            padding: 8px 14px;
            border: 1px solid rgba(123, 47, 255, 0.3);
            border-radius: 8px;
            background: rgba(0, 0, 0, 0.3);
            color: #e0e0e0;
            font-size: 13px;
            outline: none;
            width: 200px;
        }}
        .article-search:focus {{
            border-color: #b026ff;
        }}
        .article-search::placeholder {{
            color: #6666aa;
        }}

        .content-wrapper {{
            flex: 1;
            overflow-y: auto;
            padding: 26px 28px 32px;
        }}

        .content-wrapper::-webkit-scrollbar {{ width: 10px; }}
        .content-wrapper::-webkit-scrollbar-track {{ background: rgba(10, 5, 20, 0.7); }}
        .content-wrapper::-webkit-scrollbar-thumb {{ background: rgba(123, 47, 255, 0.55); border-radius: 6px; }}

        .card {{
            background: rgba(20, 15, 40, 0.92);
            border: 1px solid rgba(123, 47, 255, 0.35);
            border-radius: 16px;
            padding: 28px 30px;
            box-shadow: 0 18px 50px rgba(0, 0, 0, 0.35);
            line-height: 1.85;
        }}

        pre {{
            background: #161b22;
            padding: 18px 20px;
            border-radius: 8px;
            border: 1px solid rgba(123, 47, 255, 0.3);
            overflow-x: auto;
            font-size: 14px;
            line-height: 1.6;
        }}

        code {{
            font-family: 'Fira Code', 'Courier New', Consolas, monospace;
            color: #ff79c6;
            font-size: 14px;
        }}

        .hljs {{
            background: #161b22 !important;
            color: #c9d1d9 !important;
        }}
        .hljs-keyword {{ color: #ff7b72; }}
        .hljs-string {{ color: #a5d6ff; }}
        .hljs-number {{ color: #79c0ff; }}
        .hljs-comment {{ color: #8b949e; font-style: italic; }}
        .hljs-function, .hljs-title {{ color: #d2a8ff; }}
        .hljs-built_in, .hljs-type {{ color: #ffa657; }}

        blockquote {{
            border-left: 4px solid #b026ff;
            padding-left: 16px;
            color: #a0a0b0;
            margin: 14px 0;
        }}

        h1, h2, h3 {{ color: #b026ff; margin: 18px 0 12px; }}
        a {{ color: #b026ff; }}
        img {{ max-width: 100%; border-radius: 10px; }}
        table {{ border-collapse: collapse; width: 100%; margin: 18px 0; }}
        th, td {{ border: 1px solid rgba(123, 47, 255, 0.35); padding: 12px 14px; text-align: left; }}
        th {{ background: rgba(123, 47, 255, 0.18); color: #b026ff; }}
        tr:nth-child(even) {{ background: rgba(123, 47, 255, 0.06); }}

        .search-highlight {{
            background: #b026ff;
            color: #fff;
            padding: 1px 4px;
            border-radius: 3px;
        }}

        .footer-mark {{
            margin-top: 26px;
            padding-top: 16px;
            border-top: 1px solid rgba(123, 47, 255, 0.2);
            color: #8888aa;
            text-align: center;
            font-size: 13px;
        }}

        .sidebar li.hidden {{ display: none; }}

        .nav-separator {{
            height: 1px;
            background: rgba(123, 47, 255, 0.2);
            margin: 12px 0;
        }}
    </style>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/rust.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/python.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/javascript.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/bash.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/xml.min.js"></script>
    <script>hljs.highlightAll();</script>
</head>
<body>
    <div class="page-shell">
        <div class="sidebar" id="sidebar">
            <h3>
                📚 文档导航
                <button class="toggle-btn" onclick="toggleSidebar()">收起</button>
            </h3>
            <input type="text" class="sidebar-search" id="sidebarSearch" placeholder="搜索目录..." oninput="filterSidebar()">
            <div class="sidebar-section-title">官方</div>
            <ul id="officialLinks">
                {}
            </ul>
            <div class="nav-separator"></div>
            <div class="sidebar-section-title">Router</div>
            <ul id="routerLinks">
                {}
            </ul>
        </div>

        <div class="main-container" id="main">
            <div class="top-bar">
                <button class="expand-btn" onclick="toggleSidebar()">☰ 目录</button>
                <a href="/" class="brand">Rssust</a>
                <a href="/" class="nav-link">主页</a>
                <span class="spacer"></span>
                <div class="current-page-badge">
                    <span class="badge-icon">📄</span>
                    <span class="badge-text" id="page-title">{}</span>
                </div>
                <input type="text" class="article-search" id="articleSearch" placeholder="搜索文章内容..." oninput="searchContent()">
            </div>
            <div class="content-wrapper">
                <div class="card" id="contentCard">
                    {}
                </div>
                <div class="footer-mark">
                    2026 - Power by rust and made for Rssust
                </div>
            </div>
        </div>
    </div>

    <script>
        function toggleSidebar() {{
            const sidebar = document.getElementById('sidebar');
            const main = document.getElementById('main');
            sidebar.classList.toggle('collapsed');
            main.classList.toggle('expanded');
        }}

        function filterSidebar() {{
            const query = document.getElementById('sidebarSearch').value.toLowerCase();
            document.querySelectorAll('#officialLinks li, #routerLinks li').forEach(li => {{
                const text = li.textContent.toLowerCase();
                li.classList.toggle('hidden', !text.includes(query));
            }});
        }}

        function searchContent() {{
            const query = document.getElementById('articleSearch').value.trim().toLowerCase();
            const card = document.getElementById('contentCard');
            if (!query) {{
                if (card.dataset.originalHtml) {{
                    card.innerHTML = card.dataset.originalHtml;
                    delete card.dataset.originalHtml;
                    hljs.highlightAll();
                }}
                return;
            }}
            if (!card.dataset.originalHtml) {{
                card.dataset.originalHtml = card.innerHTML;
            }}
            const original = card.dataset.originalHtml;
            const temp = document.createElement('div');
            temp.innerHTML = original;
            const walker = document.createTreeWalker(temp, NodeFilter.SHOW_TEXT, null, false);
            const nodesToReplace = [];
            while (walker.nextNode()) {{
                const node = walker.currentNode;
                if (node.textContent.toLowerCase().includes(query)) {{
                    nodesToReplace.push(node);
                }}
            }}
            for (const node of nodesToReplace) {{
                const span = document.createElement('span');
                const escaped = query.replace(/[.*+?^${{}}()|[\]\\\\]/g, '\\\\$&');
                span.innerHTML = node.textContent.replace(
                    new RegExp('(' + escaped + ')', 'gi'),
                    '<span class="search-highlight">$1</span>'
                );
                node.parentNode.replaceChild(span, node);
            }}
            card.innerHTML = temp.innerHTML;
        }}
    </script>
</body>
</html>"#,
            file_name,
            official_links,
            router_links,
            file_name,
            html_body
        );

        let output_path = Path::new(&output_dir).join(format!("{}.html", file_name));
        fs::write(output_path, full_html)?;
    }

    Ok(())
}

//传入的像是/doc/new.html
pub fn show_doc(path:&str) -> Result<String, Error> {
    match fs::read_to_string(&Path::new(path.trim_start_matches('/'))) {
        std::result::Result::Ok(i) => Ok(i),

        Err(_) => Ok(fs::read_to_string(&Path::new("index/404.html")).context("404 html Operation failed")?),
    }
}
