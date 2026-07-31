pub mod config;
#[cfg(feature = "cookie")]
pub mod cookies;
#[cfg(feature = "docs")]
pub mod doc;
pub mod easyuser;
pub mod logger;
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
    use tracing::{debug, info, warn};

    pub fn handle_connection(mut stream: TcpStream) {
        let mut buffer: [u8; 1024] = [0; 1024];
        if let Err(e) = stream.read(&mut buffer) {
            warn!("Failed to read request: {}", e);
            return;
        }

        let head = std::str::from_utf8(extract_between_spaces(&buffer).unwrap_or_else(|| {
            warn!("Failed to extract path from request head: {:?}", &buffer[..64]);
            &[]
        }))
        .unwrap_or_else(|_| {
            warn!("Request head is not valid UTF-8: {:?}", &buffer[..64]);
            ""
        });
        let (path, params) = if let Some((before, after)) = head.split_once('?') {
            (before, crate::easyuser::params_to_hashmap(after))
        } else {
            (head, HashMap::new())
        };
        info!("Received request: {} params: {:?}", path, params);

        let response = root_rules(path, params);
        let response = match response {
            ShowToUser::Html { res } => match res {
                std::result::Result::Ok(i) => {
                    debug!("Returning HTML, length {}", i.len());
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                        i.len(),
                        i
                    )
                }
                Err(i) => {
                    warn!("Error requesting HTML: {}", i);
                    format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i)
                },
            },
            ShowToUser::Rss { res } => match res {
                std::result::Result::Ok(i) => {
                    debug!("Returning RSS, length {}", i.len());
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/xml; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                        i.len(),
                        i
                    )
                }
                Err(i) => {
                    warn!("Error requesting RSS: {}", i);
                    format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i)
                },
            },
            ShowToUser::File { res, content_type } => match res {
                std::result::Result::Ok(i) => {
                    debug!("Returning file ({}), length {}", content_type, i.len());
                    let head = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                        content_type,
                        i.len()
                    );
                    let mut body = head.into_bytes();
                    body.extend_from_slice(&i);
                    if let Err(e) = stream.write(&body).and_then(|_| stream.flush()) {
                        warn!("Failed to write response: {}", e);
                    }
                    return;
                }
                Err(i) => {
                    warn!("Error requesting file: {}", i);
                    format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i)
                },
            },
        };

        if let Err(e) = stream.write(response.as_bytes()).and_then(|_| stream.flush()) {
            warn!("Failed to write response: {}", e);
        }
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

        let mut raw = exe_dir.join(path.trim_matches('/'));
        if raw.is_dir() {
            raw = raw.join("index.html");
        }
        match fs::read_to_string(&raw) {
            std::result::Result::Ok(i) => Ok(i),

            Err(_) => Ok(fs::read_to_string(&exe_dir.join("docs/404.html"))
                .context("404 html Operation failed")?),
        }
    }

    fn read_file_bytes(path: &str) -> Result<Vec<u8>, Error> {
        let exe_path = env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("Could not get executable directory"))?;

        fs::read(exe_dir.join(path.trim_matches('/')))
            .map_err(|e| anyhow!(format!("{}:{}", path, e.kind())))
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

        if mime == "text/html; charset=utf-8" {
            return crate::request_rules::ShowToUser::Html {
                res: show_doc(path),
            };
        }

        let res = read_file_bytes(path);
        match res {
            std::result::Result::Ok(_) => crate::request_rules::ShowToUser::File {
                res,
                content_type: mime.to_string(),
            },
            std::result::Result::Err(e) => {
                warn!("Failed to read static file {}: {}", path, e);
                crate::request_rules::ShowToUser::Html {
                    res: Err(anyhow!(format!("404 html Operation failed: {}", e))),
                }
            }
        }
    }
}
