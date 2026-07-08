pub mod easyuser;
pub mod request_rules;
pub mod router;
use comrak::{markdown_to_html, ComrakOptions};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

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
        match fs::read_to_string(&Path::new("index.html")) {
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
        let raw = std::fs::read_to_string("cookies.json")?;

        let cookies: Vec<Value> = serde_json::from_str(&raw)?;
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

pub fn doc_generate() -> Result<(), Box<dyn std::error::Error>> {
    //这个函数是AI写的，我再修改，从试验项目迁到这，不要骂我
    let input_dir = "./docs_md"; // 指定文件夹
    let output_dir = "./docs";
    fs::create_dir_all(output_dir)?;

    // 1. 收集所有 md 文件路径
    let mut md_files = Vec::new();
    for entry in WalkDir::new(input_dir) {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
            md_files.push(entry.path().to_path_buf());
        }
    }

    // 2. 生成全局导航链接 (美化部分：侧边栏)
    let mut nav_links = String::new();
    for path in &md_files {
        let file_name = path.file_stem().unwrap().to_str().unwrap();
        nav_links.push_str(&format!("<li><a href='{}.html'>{}</a></li>", file_name, file_name));
    }

    // 3. 逐个转换
    for path in &md_files {
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

        .sidebar {{
            flex: 0 0 260px;
            width: 260px;
            background: rgba(10, 5, 20, 0.96);
            padding: 22px;
            border-right: 1px solid rgba(123, 47, 255, 0.35);
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

        .top-bar h2 {{
            font-size: 22px;
            color: #b026ff;
            margin: 0;
            word-break: break-word;
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

        .footer-mark {{
            margin-top: 26px;
            padding-top: 16px;
            border-top: 1px solid rgba(123, 47, 255, 0.2);
            color: #8888aa;
            text-align: center;
            font-size: 13px;
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
            <ul>
                {}
            </ul>
        </div>

        <div class="main-container" id="main">
            <div class="top-bar">
                <button class="expand-btn" onclick="toggleSidebar()">展开/收起目录</button>
                <h2>{}</h2>
            </div>
            <div class="content-wrapper">
                <div class="card">
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
    </script>
</body>
</html>"#,
            file_name,
            nav_links,
            file_name,
            html_body
        );

        let output_path = Path::new(output_dir).join(format!("{}.html", file_name));
        fs::write(output_path, full_html)?;
    }

    Ok(())
}
