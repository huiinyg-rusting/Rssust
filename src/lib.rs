///这个函数提供缓冲区的处理
/// 并把数据交给request_rules函数处理
/// 最终在这个函数体内发送http数据
pub mod connect {
    use anyhow::Error;
    use anyhow::{Result, anyhow};

    use std::fs;
    use std::io::prelude::*;
    use std::net::TcpStream;
    use std::path::Path;

    use crate::crawler::{fetch_obscura, fetch_reqwest_get, fetch_reqwest_post};

    pub fn handle_connection(mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let response = request_rules(buffer); //.expect("在加载request_rules时发生错误");
        let response = match response {
            Ok(i) => format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                i.len(),
                i
            ),
            Err(i) => format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i),
        };

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    ///这个函数相当于模块的注册表
    /// 给调用者的是html格式
    fn request_rules(buffer: [u8; 1024]) -> Result<String, Error> {
        if buffer.starts_with(b"GET / ") {
            show_index_doc()
        } else if buffer.starts_with(b"GET /what ") {
            Ok("What is this?".to_string())
        } 
        else if buffer.starts_with(b"GET /bilibili ") {
            fetch_obscura("https://bilibili.com")
        } else if buffer.starts_with(b"GET /git ") {
            fetch_reqwest_get("https://api.kuleu.com/api/getGreetingMessage?type=json")
        } else if buffer.starts_with(b"GET /git1 ") {
            fetch_reqwest_post("http://is.snssdk.com/api/news/feed/v51/", "".to_string())
        }
        else {
            Err(anyhow!("404NotFound"))
        }
    }

    ///这个函数发送index.html的内容给调用者，否则发送错误及anyhow的文本错误类型 给调用者
    /// 给调用者的是html格式
    fn show_index_doc() -> Result<String, Error> {
        match fs::read_to_string(&Path::new("index.html")) {
            Ok(i) => Ok(i),

            Err(i) => Err(anyhow!(format!("{}:{}", "index.html", i.kind()))),
        }
    }
}


pub mod crawler {
    use std::cell::RefCell;
    use std::time::{SystemTime, UNIX_EPOCH};
    use anyhow::{ Error, Ok, Result, anyhow};
    use obscura::Browser;
    use serde_json::Value;
    use tokio::runtime::Builder as RuntimeBuilder;
    use tokio::runtime::Runtime;

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
                let rt = RuntimeBuilder::new_current_thread()
                    .enable_all()
                    .build()?;
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
        if cookie.get("secure").and_then(Value::as_bool).unwrap_or(false) {
            set_cookie.push_str("; Secure");
        }
        if cookie.get("httpOnly").and_then(Value::as_bool).unwrap_or(false) {
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

        Ok(Coke { url: (domain.to_string()), mai: (set_cookie) })


        }


    /// 从 JSON 文件加载 cookie，返回 Set-Cookie 字符串列表。
    /// JSON 文件应为 cookie 对象数组。
    pub fn load_cookies() -> Result<String,Error> {
        let raw = std::fs::read_to_string("cookies.json")?;

        let cookies: Vec<Value> = serde_json::from_str(&raw)?;
        let coke_list: Vec<Coke> = cookies.iter()
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
                browser.cookies().set(cmake.mai.as_str(),format!("https://www{}",cmake.url).as_str())?;
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

    /// 从指定 URL 抓取 HTML。
    /// 推荐
    pub fn fetch_obscura(url:&str) -> Result<String,Error>{
        let html = fetch(url)?;
        println!("[OK] {} ({} bytes)\n", url, html.len());
        Ok(html)
    }

    //下面是reqwest get的内容
    //不会使用线程池
    pub fn fetch_reqwest_get(url: &str) -> Result<String,Error> {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
        Ok(reqwest::get(url).await?.text().await?)
        })
    }

    ///This can be an array of tuples, or a HashMap, or a custom type that implements Serialize.
    ///这可以是一个元组数组，或者是一个 HashMap ，或者是一个实现了 Serialize 的自定义类型。
    ///The feature form is required.
    ///必须使用 form 功能
    pub fn fetch_reqwest_post(url: &str,body: String) -> Result<String,Error>{
        let rt = Runtime::new().unwrap();
            rt.block_on(async {
            Ok(reqwest::Client::new().post(url).body(body).send().await?.text().await?)
        })
    }
}
