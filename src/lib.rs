pub mod cookies;
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
    use log::warn;
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
            warn!("Failed to slice header{:?}", buffer);
            &[]
        }))
        .unwrap_or_else(|_| {
            warn!("Invalid UTF-8{:?}", buffer);
            ""
        });
        let (path, params) = if let Some((before, after)) = head.split_once('?') {
            (before, crate::easyuser::params_to_hashmap(after))
        } else {
            (head, HashMap::new())
        };

        let response = root_rules(path, params);
        let response = match response {
            ShowToUser::Html { res } => match res {
                std::result::Result::Ok(i) => format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                    i.len(),
                    i
                ),
                Err(i) => {
                    warn!("‌Client error occurred when requesting HTML. Error details: {}", i);
                    format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i)
            },
            },
            ShowToUser::Rss { res } => match res {
                std::result::Result::Ok(i) => format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/xml; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                    i.len(),
                    i
                ),
                Err(i) => {
                    warn!("Client error occurred when requesting XML. Error details:‌ {}", i);
                    format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i)
                },
            },
            ShowToUser::File { res, content_type } => match res {
                std::result::Result::Ok(i) => format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                    content_type,
                    i.len(),
                    i
                ),
                Err(i) => {
                    warn!("Client error occurred when requesting data (file). Error details:‌ {}", i);
                    format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i)},
            },
        };

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn extract_between_spaces(buffer: &[u8; 1024]) -> Option<&[u8]> {
        let bytes = buffer.as_slice();

        // Find the position of the first space.‌
        let first_space = bytes.iter().position(|&b| b == b' ')?;

        // Extract the content between the first and second spaces.‌
        let second_space = bytes[first_space + 1..]
            .iter()
            .position(|&b| b == b' ')
            .map(|pos| first_space + 1 + pos)?;

        // Extract the content between two spaces.‌
        Some(&bytes[first_space + 1..second_space])
    }

    ///This function sends the content of index.html to the caller; otherwise, it sends an error with an anyhow text error type.‌
    /// The response returned to the caller is in HTML format.
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

    pub fn mime_type(path: &str) -> &'static str {
        if path.ends_with(".css") {
            "text/css; charset=utf-8"
        } else if path.ends_with(".js") {
            "application/javascript; charset=utf-8"
        } else if path.ends_with(".svg") {
            "image/svg+xml"
        } else if path.ends_with(".png") {
            "image/png"
        } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
            "image/jpeg"
        } else if path.ends_with(".gif") {
            "image/gif"
        } else if path.ends_with(".ico") {
            "image/x-icon"
        } else if path.ends_with(".woff2") {
            "font/woff2"
        } else if path.ends_with(".woff") {
            "font/woff"
        } else if path.ends_with(".ttf") {
            "font/ttf"
        } else if path.ends_with(".json") {
            "application/json; charset=utf-8"
        } else if path.ends_with(".xml") {
            "application/xml; charset=utf-8"
        } else {
            "text/html; charset=utf-8"
        }
    }

    pub fn serve_static(path: &str) -> crate::request_rules::ShowToUser {
        let mime = mime_type(path);
        let res = show_doc(path);

        if mime == "text/html; charset=utf-8" {
            return crate::request_rules::ShowToUser::Html { res };
        }

        let file_exists = match env::current_exe() {
            std::result::Result::Ok(p) => match p.parent() {
                Some(d) => d.join(path.trim_matches('/')).exists(),
                None => false,
            },
            std::result::Result::Err(_) => false,
        };

        if file_exists {
            crate::request_rules::ShowToUser::File {
                res,
                content_type: mime.to_string(),
            }
        } else {
            crate::request_rules::ShowToUser::Html { res }
        }
    }
}

pub mod crawler {
    use anyhow::{Error, Ok, Result, anyhow};
    use obscura::Browser;
    use serde_json::Value;
    use std::cell::RefCell;
    use std::env;
    use log::info;
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

    pub fn with_browser<F, T>(f: F) -> Result<T>
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

    /// Convert a JSON cookie object to an HTTP Set-Cookie string.‌
    ///
    /// e.g：
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
            .ok_or_else(|| anyhow!("cookie need name field‌"))?;
        let value = cookie
            .get("value")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("cookie need value field‌"))?;
        let domain = cookie
            .get("domain")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("cookie need domain field‌"))?;
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

    /// Load cookies from a JSON file and return a list of Set-Cookie strings.‌
    /// The JSON file should be an array of cookie objects.‌
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
        info!("Succse {} Cookie", cookies.len());
        Ok("Succse".to_string())
    }

    /// Inject cookies into the global browser instance.‌
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

}
