pub mod doc;
pub mod easyuser;
pub mod request_rules;
pub mod router;

///这个函数提供缓冲区的处理
/// 并把数据交给request_rules函数处理
/// 最终在这个函数体内发送http数据
pub mod connect {
    use crate::request_rules::*;
    use anyhow::Error;
    use anyhow::*;
    use std::collections::HashMap;
    use std::env;
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
            ShowToUser::Html { res } => match res {
                std::result::Result::Ok(i) => format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                    i.len(),
                    i
                ),
                Err(i) => format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i),
            },
            ShowToUser::Rss { res } => match res {
                std::result::Result::Ok(i) => format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/xml; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                    i.len(),
                    i
                ),
                Err(i) => format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i),
            },
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
        let exe_path = env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("Could not get executable directory"))?;

        match fs::read_to_string(&Path::new(&exe_dir.join("index/index.html"))) {
            std::result::Result::Ok(i) => Ok(i),

            Err(i) => Err(anyhow!(format!("{}:{}", "index.html", i.kind()))),
        }
    }
    //传入的像是/doc/new.html
    pub fn show_doc(path: &str) -> Result<String, Error> {
        let exe_path = env::current_exe()?;

        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("Could not get executable directory"))?;

        let raw = exe_dir.join(path.trim_matches('/'));
        match fs::read_to_string(raw) {
            std::result::Result::Ok(i) => Ok(i),

            Err(_) => Ok(fs::read_to_string(&Path::new("index/404.html"))
                .context("404 html Operation failed")?),
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

        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("Could not get executable directory"))?;

        let raw = exe_dir.join("cookies.json");
        let text = std::fs::read_to_string(raw)?;
        if text.is_empty() {
            return Ok("".to_string());
        };
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
